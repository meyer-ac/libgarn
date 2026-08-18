use crate::interface::error_handling::PartialError;
use crate::{ffi_partial_error, ffi_partial_error_with_details};
use garnshared::linux::traits::ShmSync;
use hashed_type_def::HashedTypeMethods;
use nix::sys::mman::{MapFlags, ProtFlags, mmap};
use nix::unistd::{SysconfVar, sysconf};
use std::collections::HashMap;
use std::num::NonZero;
use std::os::fd::{AsFd, FromRawFd, OwnedFd, RawFd};
use std::ptr::NonNull;

struct ResourceMetadata {
    page: usize,
    offset: usize,
}

struct Page {
    fd: OwnedFd,
    mem: NonNull<u8>,
}

pub struct ShmConsumer {
    page_size: usize,
    resources: HashMap<String, ResourceMetadata>,
    pages: Vec<Option<Page>>,
}

impl ShmConsumer {
    pub fn new() -> Result<Self, PartialError> {
        let page_size = match sysconf(SysconfVar::PAGE_SIZE) {
            Ok(Some(0) | None) | Err(_) => return Err(ffi_partial_error!(GetPageSizeFailed)),
            Ok(Some(res)) => usize::try_from(res).unwrap(), // non-negative according to the Linux kernel
        };

        Ok(Self {
            page_size,
            resources: HashMap::new(),
            pages: Vec::new(),
        })
    }

    /// # SAFETY
    /// * The resource pointed to by `fd` must be open and the size of a memory page.
    /// * The resource pointed to by `fd` must be either already consumed by this object or suitable for assuming ownership.
    /// * The resource pointed to by `fd` must not require any cleanup other than close.
    /// * The consumed resource must be of type `T`
    pub unsafe fn consume<T: ShmSync>(
        &mut self,
        name: &str,
        page_fd: RawFd,
        page: usize,
        offset: usize,
    ) -> Result<&T, PartialError> {
        if self.resources.contains_key(name) {
            // SAFETY: guaranteed by function invariants
            return Ok(unsafe { self.access_resource(page, offset) });
        }
        if self.pages.len() <= page || self.pages[page].is_none() {
            // SAFETY: guaranteed by function invariants
            unsafe { self.load_page(page_fd, page) }?;
        }
        self.resources
            .insert(name.to_owned(), ResourceMetadata { page, offset });
        // SAFETY: guaranteed by function invariants
        Ok(unsafe { self.access_resource(page, offset) })
    }

    /// SAFETY:
    /// Accessed resource must be of type `T`.
    unsafe fn access_resource<T: ShmSync>(&self, page: usize, offset: usize) -> &T {
        unsafe {
            &*self.pages[page]
                .as_ref()
                .unwrap()
                .mem
                .as_ptr()
                .add(offset)
                .cast::<T>()
        }
    }

    /// # SAFETY
    /// * The resource pointed to by `fd` must be open and the size of a memory page.
    /// * The resource pointed to by `fd` must be suitable for assuming ownership.
    /// * The resource pointed to by `fd` must not require any cleanup other than close.
    unsafe fn load_page(&mut self, fd: RawFd, dest: usize) -> Result<(), PartialError> {
        if self.pages.len() <= dest {
            self.pages.reserve(dest - self.pages.len() + 1);
            for _ in self.pages.len()..=dest {
                self.pages.push(None);
            }
        }
        // SAFETY: guaranteed by function invariants
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        // SAFETY: length is guaranteed to be non-zero in Self::new(),
        // prot and flags are only passed valid flags,
        // offset is trivially a multiple of the system's page size and
        // addr is omitted.
        match unsafe {
            mmap(
                None,
                NonZero::new(self.page_size).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                owned_fd.as_fd(),
                0,
            )
        } {
            Ok(res) => {
                self.pages[dest] = Some(Page {
                    fd: owned_fd,
                    mem: res.cast::<u8>(),
                });
            }
            Err(e) => {
                return Err(ffi_partial_error_with_details!(
                    SharedMemoryError,
                    e.to_string()
                ));
            }
        }

        Ok(())
    }
}
