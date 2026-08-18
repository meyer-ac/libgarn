use crate::ffi_partial_error_with_details;
use crate::interface::error_handling::PartialError;
use crate::linux::mutex::Mutex;
use crate::linux::shm_consumer::ShmConsumer;
use crate::platform_traits::PlatformEnvironment;
use garnshared::constants::{
    ENVIRONMENT_REQUEST_SIZE, ENVIRONMENT_RESPONSE_SIZE, MAX_NAME_LEN, WELCOME_RESPONSE_SIZE,
};
use garnshared::environment_protocol::{EnvironmentRequest, EnvironmentResponse};
use garnshared::error_types::SerializeError;
use garnshared::linux::pthread_mutex::PthreadMutex;
use garnshared::welcome_protocol::{WelcomeRequest, WelcomeResponse};
use nix::cmsg_space;
use nix::sys::socket::AddressFamily::Unix;
use nix::sys::socket::SockType::SeqPacket;
use nix::sys::socket::{
    ControlMessageOwned, MsgFlags, SockFlag, UnixAddr, connect, recv, recvmsg, send, socket,
};
use std::collections::HashMap;
use std::io::IoSliceMut;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::thread::{self, ThreadId};

pub struct Environment {
    owner_thread: ThreadId,
    name: String,
    open_mutexes: HashMap<String, *const Mutex>,
    socket: OwnedFd,
    shm_consumer: ShmConsumer,
}

impl PlatformEnvironment for Environment {
    fn new(name: &str) -> Result<Self, PartialError> {
        let shm_consumer = ShmConsumer::new()?;

        let socket = match socket(Unix, SeqPacket, SockFlag::SOCK_CLOEXEC, None) {
            Ok(res) => res,
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    e.to_string()
                ));
            }
        };

        let welcome_sock_name = String::from_iter([
            garnshared::constants::ABSTRACT_SOCK_NAME_PREFIX,
            garnshared::constants::WELCOME_SOCK_ABSTRACT_NAME,
        ]);

        let addr = match UnixAddr::new_abstract(welcome_sock_name.as_bytes()) {
            Ok(res) => res,
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    e.to_string()
                ));
            }
        };

        if let Err(e) = connect(socket.as_raw_fd(), &addr) {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                e.to_string()
            ));
        }

        let request = match WelcomeRequest::OpenEnvironment(name.to_owned()).serialize() {
            Ok(res) => res,
            Err(SerializeError::NameTooLongError) => {
                return Err(ffi_partial_error_with_details!(
                    NameTooLong,
                    format!(
                        "The maximum length of an environment name is {} bytes.",
                        MAX_NAME_LEN
                    )
                ));
            }
        };

        if let Err(e) = send(socket.as_raw_fd(), request.as_bytes(), MsgFlags::empty()) {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                e.to_string()
            ));
        }

        let mut buffer: [u8; WELCOME_RESPONSE_SIZE] = [0; WELCOME_RESPONSE_SIZE];

        if let Err(e) = recv(socket.as_raw_fd(), &mut buffer, MsgFlags::empty()) {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                e.to_string()
            ));
        }

        let response_str = match String::from_utf8(buffer.to_vec()) {
            Ok(res) => res,
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    e.to_string()
                ));
            }
        };

        let Some(response) = WelcomeResponse::deserialize(&response_str) else {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                String::from("Deserialization of the service response failed.")
            ));
        };

        match response {
            WelcomeResponse::MalformedRequest => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    String::from("garnd service reported a malformed request.")
                ));
            }
            WelcomeResponse::InternalError => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    String::from("garnd service reported an internal error.")
                ));
            }
            WelcomeResponse::OpenEnvironmentOk => (),
        }

        Ok(Self {
            owner_thread: thread::current().id(),
            name: name.into(),
            open_mutexes: HashMap::new(),
            socket,
            shm_consumer,
        })
    }

    fn get_owner_thread(&self) -> ThreadId {
        self.owner_thread
    }

    fn open_mutex(&mut self, name: &str) -> Result<*const Mutex, PartialError> {
        if let Some(&mutex) = self.open_mutexes.get(name) {
            return Ok(mutex);
        }

        let request = match EnvironmentRequest::OpenMutex(name.to_owned()).serialize() {
            Ok(res) => res,
            Err(SerializeError::NameTooLongError) => {
                return Err(ffi_partial_error_with_details!(
                    NameTooLong,
                    format!(
                        "The maximum length of a mutex name is {} bytes.",
                        MAX_NAME_LEN
                    )
                ));
            }
        };

        if let Err(e) = send(
            self.socket.as_raw_fd(),
            request.as_bytes(),
            MsgFlags::empty(),
        ) {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                e.to_string()
            ));
        }

        let mut buffer: [u8; ENVIRONMENT_RESPONSE_SIZE] = [0; ENVIRONMENT_RESPONSE_SIZE];
        let mut iov = [IoSliceMut::new(&mut buffer)];
        let mut cmsg_buffer = cmsg_space!([RawFd; 1]);

        let msgs = match recvmsg::<()>(
            self.socket.as_raw_fd(),
            &mut iov,
            Some(&mut cmsg_buffer),
            MsgFlags::empty(),
        ) {
            Ok(res) => res,
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    e.to_string()
                ));
            }
        };

        let cmsgs = match msgs.cmsgs() {
            Ok(res) => res,
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    e.to_string()
                ));
            }
        };

        let mut shm_fd = None;
        for cmsg in cmsgs {
            if let ControlMessageOwned::ScmRights(fds) = cmsg
                && let Some(&fd) = fds.first()
            {
                shm_fd = Some(fd);
                break;
            }
        }

        if shm_fd.is_none() {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                String::from("The garnd service did not provide a shared memory file descriptor.")
            ));
        }

        let response_str = match String::from_utf8(buffer.to_vec()) {
            Ok(res) => res,
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    e.to_string()
                ));
            }
        };

        let Some(response) = EnvironmentResponse::deserialize(&response_str) else {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                String::from("Deserialization of the service response failed.")
            ));
        };

        let (page, offset) = match response {
            EnvironmentResponse::MalformedRequest => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    String::from("garnd service reported a malformed request.")
                ));
            }
            EnvironmentResponse::InternalError => {
                return Err(ffi_partial_error_with_details!(
                    ServiceCommunicationFailed,
                    String::from("garnd service reported an internal error.")
                ));
            }
            EnvironmentResponse::OpenMutexOk(page, offset) => (page, offset),
        };

        // Safety: shm_fd was just obtained from the socket (is open) and no one else will use it
        // (safe to assume ownership if not already consumed),
        // the garnd service is trusted to provide an object of the right type and a memory page
        // of the right size with trivial cleanup.
        // Cast is safe because of repr(transparent) on Mutex.
        let mutex_ptr = unsafe {
            self.shm_consumer
                .consume::<PthreadMutex>(name, shm_fd.unwrap(), page, offset)?
        }
        .cast::<Mutex>();
        self.open_mutexes.insert(name.to_owned(), mutex_ptr);
        Ok(mutex_ptr)
    }
}
