use crate::alive_marker::AliveMarker;
use crate::ffi_partial_error_with_details;
use crate::interface::error_handling::PartialError;
use crate::linux::mutex::Mutex;
use crate::linux::shm_consumer::ShmConsumer;
use crate::platform_traits::PlatformEnvironment;
use garnshared::constants::MAX_NAME_LEN;
use garnshared::environment_protocol::{
    ENVIRONMENT_RESPONSE_PROTOCOL, EnvironmentRequest, EnvironmentResponse,
};
use garnshared::linux::pthread_mutex::PthreadMutex;
use garnshared::message_parser::MessageProtocolError;
use garnshared::welcome_protocol::{WELCOME_RESPONSE_PROTOCOL, WelcomeRequest, WelcomeResponse};
use nix::cmsg_space;
use nix::sys::socket::AddressFamily::Unix;
use nix::sys::socket::SockType::SeqPacket;
use nix::sys::socket::{
    ControlMessageOwned, MsgFlags, SockFlag, UnixAddr, connect, recv, recvmsg, send, socket,
};
use std::collections::HashMap;
use std::io::IoSliceMut;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::thread::{self, ThreadId};

pub struct Environment {
    owner_thread: ThreadId,
    open_mutexes: HashMap<String, *const Mutex>,
    socket: OwnedFd,
    shm_consumer: ShmConsumer,
    alive_marker: AliveMarker,
}

impl PlatformEnvironment for Environment {
    #[allow(refining_impl_trait)]
    fn new(name: &str) -> Result<Self, PartialError> {
        let shm_consumer = ShmConsumer::new()?;

        let socket = socket(Unix, SeqPacket, SockFlag::SOCK_CLOEXEC, None).map_err(|e| {
            ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string())
        })?;

        let welcome_sock_name = String::from_iter([
            garnshared::constants::ABSTRACT_SOCK_NAME_PREFIX,
            garnshared::constants::WELCOME_SOCK_ABSTRACT_NAME,
        ]);

        let addr = UnixAddr::new_abstract(welcome_sock_name.as_bytes()).map_err(|e| {
            ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string())
        })?;

        if let Err(e) = connect(socket.as_raw_fd(), &addr) {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                e.to_string()
            ));
        }

        let request = match WelcomeRequest::OpenEnvironment(name.to_owned()).serialize() {
            Ok(res) => res,
            Err(MessageProtocolError::ArgumentTooLong {
                mnemonic: _,
                argument_number: _,
                max_len: _,
                actual_len: _,
            }) => {
                return Err(ffi_partial_error_with_details!(
                    NameTooLong,
                    format!(
                        "The maximum length of an environment name is {} bytes.",
                        MAX_NAME_LEN
                    )
                ));
            }
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    SerializationError,
                    e.to_string()
                ));
            }
        };

        if let Err(e) = send(socket.as_raw_fd(), request.as_bytes(), MsgFlags::empty()) {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                e.to_string()
            ));
        }

        let mut buffer = vec![0u8; WELCOME_RESPONSE_PROTOCOL.max_size()].into_boxed_slice();

        if let Err(e) = recv(socket.as_raw_fd(), &mut buffer, MsgFlags::empty()) {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                e.to_string()
            ));
        }

        let response_str = str::from_utf8(&buffer).map_err(|e| {
            ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string())
        })?;

        let response = WelcomeResponse::deserialize(response_str).map_err(|e| {
            ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string())
        })?;

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
            open_mutexes: HashMap::new(),
            socket,
            shm_consumer,
            alive_marker: AliveMarker::new(),
        })
    }

    fn check_alive(&self) {
        self.alive_marker.check();
    }

    fn get_owner_thread(&self) -> ThreadId {
        self.owner_thread
    }

    #[allow(refining_impl_trait)]
    fn open_mutex(&mut self, name: &str) -> Result<*const Mutex, PartialError> {
        if let Some(&mutex) = self.open_mutexes.get(name) {
            return Ok(mutex);
        }

        let request = EnvironmentRequest::OpenMutex(name.to_owned())
            .serialize()
            .map_err(|e| match e {
                MessageProtocolError::ArgumentTooLong {
                    mnemonic: _,
                    argument_number: _,
                    max_len: _,
                    actual_len: _,
                } => ffi_partial_error_with_details!(
                    NameTooLong,
                    format!(
                        "The maximum length of a mutex name is {} bytes.",
                        MAX_NAME_LEN
                    )
                ),
                e => ffi_partial_error_with_details!(SerializationError, e.to_string()),
            })?;

        send(
            self.socket.as_raw_fd(),
            request.as_bytes(),
            MsgFlags::empty(),
        )
        .map_err(|e| ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string()))?;

        let mut buffer = vec![0u8; ENVIRONMENT_RESPONSE_PROTOCOL.max_size()].into_boxed_slice();
        let mut iov = [IoSliceMut::new(&mut buffer)];
        let mut cmsg_buffer = cmsg_space!([RawFd; 1]);

        let msgs = recvmsg::<()>(
            self.socket.as_raw_fd(),
            &mut iov,
            Some(&mut cmsg_buffer),
            MsgFlags::empty(),
        )
        .map_err(|e| ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string()))?;

        let cmsgs = msgs.cmsgs().map_err(|e| {
            ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string())
        })?;

        let mut shm_fd = None;
        for cmsg in cmsgs {
            if let ControlMessageOwned::ScmRights(fds) = cmsg
                && let Some(&fd) = fds.first()
            {
                // Safety: fd was deliberately passed to us via the socket
                shm_fd = Some(unsafe { OwnedFd::from_raw_fd(fd) });
                break;
            }
        }

        let response_str = str::from_utf8(&buffer).map_err(|e| {
            ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string())
        })?;

        let response = EnvironmentResponse::deserialize(response_str).map_err(|e| {
            ffi_partial_error_with_details!(ServiceCommunicationFailed, e.to_string())
        })?;

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

        if shm_fd.is_none() {
            return Err(ffi_partial_error_with_details!(
                ServiceCommunicationFailed,
                String::from("The garnd service did not provide a shared memory file descriptor.")
            ));
        }

        // Safety: shm_fd was just obtained from the socket (is open) and no one else will use it
        // (safe to assume ownership if not already consumed),
        // the garnd service is trusted to provide an object of the right type and a memory page
        // of the right size with trivial cleanup.
        // Cast is safe because of repr(transparent) on Mutex.
        let mutex_ptr = unsafe {
            self.shm_consumer
                .consume::<PthreadMutex>(shm_fd.unwrap(), page, offset)?
        }
        .cast::<Mutex>();
        self.open_mutexes.insert(name.to_owned(), mutex_ptr);
        Ok(mutex_ptr)
    }
}
