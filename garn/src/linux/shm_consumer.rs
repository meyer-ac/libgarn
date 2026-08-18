use crate::interface::error_handling::PartialError;
use crate::util::warn;
use crate::{ffi_partial_error, ffi_partial_error_with_details};
use garnshared::linux::traits::ShmSync;
use hashed_type_def::HashedTypeMethods;
use nix::sys::mman::{MapFlags, ProtFlags, mmap, munmap};
use nix::unistd::{SysconfVar, sysconf};
use std::collections::HashMap;
use std::ffi::c_void;
use std::num::NonZero;
use std::os::fd::{AsFd, FromRawFd, OwnedFd, RawFd};
use std::ptr::NonNull;

struct Page {
    fd: OwnedFd,
    mem: NonNull<u8>,
}

pub struct ShmConsumer {
    page_size: usize,
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
    ) -> Result<*const T, PartialError> {
        if self.pages.len() <= page || self.pages[page].is_none() {
            // SAFETY: guaranteed by function invariants
            unsafe { self.load_page(page_fd, page) }?;
        }
        // SAFETY: guaranteed by function invariants
        Ok(unsafe { self.access_resource(page, offset) })
    }

    /// SAFETY:
    /// Accessed resource must be of type `T`.
    unsafe fn access_resource<T: ShmSync>(&self, page: usize, offset: usize) -> *const T {
        unsafe {
            &raw const *self.pages[page]
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

impl Drop for ShmConsumer {
    fn drop(&mut self) {
        for page in self.pages.drain(..) {
            // SAFETY: addr being a multiple of the page size is guaranteed by mmap, which
            // aligns the memory to page boundaries
            if let Some(Err(e)) =
                page.map(|page| unsafe { munmap(page.mem.cast::<c_void>(), self.page_size) })
            {
                warn(format!("unmapping of shared memory failed: {e}").as_str());
            }
        }
    }
}
