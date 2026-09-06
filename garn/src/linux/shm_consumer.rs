use crate::interface::error_handling::PartialError;
use crate::util::warn;
use crate::{ffi_partial_error, ffi_partial_error_with_details};
use garnshared::linux::traits::ShmCompatible;
use nix::sys::mman::{MapFlags, ProtFlags, mmap, munmap};
use nix::sys::stat::fstat;
use nix::unistd::{SysconfVar, sysconf};
use std::collections::HashMap;
use std::collections::hash_map::{Entry, VacantEntry};
use std::ffi::c_void;
use std::num::NonZero;
use std::os::fd::{AsFd, OwnedFd};
use std::ptr::NonNull;

struct Page {
    _fd: OwnedFd,
    mem: NonNull<u8>,
}

pub struct ShmConsumer {
    page_size: usize,
    pages: HashMap<usize, Page>,
}

impl ShmConsumer {
    pub fn new() -> Result<Self, PartialError> {
        let page_size = match sysconf(SysconfVar::PAGE_SIZE) {
            Ok(Some(0) | None) | Err(_) => return Err(ffi_partial_error!(GetPageSizeFailed)),
            Ok(Some(res)) => usize::try_from(res).unwrap(), // non-negative according to the Linux kernel
        };

        Ok(Self {
            page_size,
            pages: HashMap::new(),
        })
    }

    /// # SAFETY
    /// * The resource pointed to by `fd` must be open and the size of a memory page.
    /// * The resource pointed to by `fd` must be either already consumed by this object or suitable for assuming ownership.
    /// * The resource pointed to by `fd` must not require any cleanup other than close.
    /// * The consumed resource must be of type `T`
    pub unsafe fn consume<T: ShmCompatible>(
        &mut self,
        page_fd: OwnedFd,
        page: usize,
        offset: usize,
    ) -> Result<*const T, PartialError> {
        if let Entry::Vacant(entry) = self.pages.entry(page) {
            Self::load_page(self.page_size, page_fd, entry)?;
        }
        // SAFETY: guaranteed by function invariants
        Ok(unsafe { self.access_resource(page, offset)? })
    }

    /// SAFETY:
    /// Accessed resource must exist and be of type `T`.
    unsafe fn access_resource<T: ShmCompatible>(
        &self,
        page: usize,
        offset: usize,
    ) -> Result<*const T, PartialError> {
        if offset + size_of::<T>() > self.page_size {
            return Err(ffi_partial_error!(ShmAccessOutOfBounds));
        }
        if !offset.is_multiple_of(align_of::<T>()) {
            return Err(ffi_partial_error!(ShmMisalignedAccess));
        }
        // SAFETY: add: offset fits into isize, because the upper half of addresses is reserved for kernel space and
        // the whole range between the original address and the offset address belongs to the same
        // allocation (anonymous file). The address does also not wrap around the address space,
        // because the whole file is guaranteed to be in the lower half of the address space.
        Ok(unsafe {
            self.pages
                .get(&page)
                .unwrap()
                .mem
                .as_ptr()
                .add(offset)
                .cast::<T>()
        })
    }

    fn load_page(
        page_size: usize,
        fd: OwnedFd,
        dest: VacantEntry<usize, Page>,
    ) -> Result<(), PartialError> {
        let stats = fstat(fd.as_fd())
            .map_err(|e| ffi_partial_error_with_details!(SharedMemoryError, e.to_string()))?;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if stats.st_size as usize != page_size {
            return Err(ffi_partial_error_with_details!(
                SharedMemoryError,
                "Size of received page file does not match the system's page size".to_owned()
            ));
        }

        // SAFETY: length is guaranteed to be non-zero in Self::new(),
        // prot and flags are only passed valid flags,
        // offset is trivially a multiple of the system's page size and
        // addr is omitted.
        match unsafe {
            mmap(
                None,
                NonZero::new(page_size).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                fd.as_fd(),
                0,
            )
        } {
            Ok(res) => {
                dest.insert(Page {
                    _fd: fd,
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
        for (_, page) in self.pages.drain() {
            // SAFETY: addr being a multiple of the page size is guaranteed by mmap, which
            // aligns the memory to page boundaries
            if let Err(e) = unsafe { munmap(page.mem.cast::<c_void>(), self.page_size) } {
                warn(format!("unmapping of shared memory failed: {e}").as_str());
            }
        }
    }
}
