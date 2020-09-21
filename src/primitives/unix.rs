/*
 * garbage-collected memory manager in Rust
 * Copyright (C) 2020  Xie Ruifeng
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! memory allocation primitives for UNIX

#![cfg(unix)]

use super::MMapError;
use super::Result;

use enumflags2::BitFlags;
use libc::{c_int, c_void, off_t};

/// memory protection flags
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, BitFlags)]
pub enum Protection {
    /// Pages may be read
    Read = libc::PROT_READ as u32,
    /// Pages may be written
    Write = libc::PROT_WRITE as u32,
    /// Pages may be executed
    Exec = libc::PROT_EXEC as u32,
}

impl Protection {
    /// Pages may not be accessed
    #[allow(dead_code)]
    pub const NONE: BitFlags<Protection> = unsafe { core::mem::transmute(0u32) };
}

/// mmap flags
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, BitFlags)]
pub enum MapFlags {
    /// Share this mapping (updates visible to other processes)
    Shared = libc::MAP_SHARED as u32,
    /// Private copy-on-write mapping
    Private = libc::MAP_PRIVATE as u32,
    /// The mapping is not backed by any file; its contents are initialized to zero
    Anonymous = libc::MAP_ANONYMOUS as u32,
    /// Do not reserve swap space for this mapping
    NoReserve = libc::MAP_NORESERVE as u32,
}

const INVALID_FILE_DESCRIPTOR: libc::c_int = -1;

unsafe fn wrapped_mmap(
    addr: *mut c_void, len: usize,
    prot: BitFlags<Protection>, flags: BitFlags<MapFlags>,
    fd: c_int, offset: off_t) -> *mut c_void {
    libc::mmap(addr, len, prot.bits() as c_int, flags.bits() as c_int, fd, offset)
}

// the following copied from nix
#[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "android"))]
use libc::__errno as errno_location;
#[cfg(any(target_os = "linux", target_os = "emscripten", target_os = "redox"))]
use libc::__errno_location as errno_location;
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use libc::___errno as errno_location;
#[cfg(any(target_os = "macos", target_os = "freebsd"))]
use libc::__error as errno_location;
#[cfg(target_os = "haiku")]
use libc::_errnop as errno_location;

unsafe fn get_errno() -> c_int { *errno_location() }

unsafe fn set_errno(e: c_int) { *errno_location() = e; }

impl MMapError {
    /// get `MMapError` from an `errno` value
    pub fn from_errno(e: c_int) -> MMapError {
        match e {
            libc::EINVAL => MMapError::InvalidArguments,
            libc::EAGAIN => MMapError::TryAgain,
            libc::ENOMEM => MMapError::NoMemory,
            libc::EOVERFLOW => MMapError::LengthOverflow,
            0 => MMapError::NoError,
            _ => MMapError::UnknownError(e as u32),
        }
    }

    /// get `MMapError` from the current `errno` value
    pub unsafe fn get() -> MMapError {
        Self::from_errno(get_errno())
    }
}

/// According to the Linux manual for `_SC_PAGESIZE`:
///   Size of a page in bytes.  Must not be less than 1.
static mut PAGE_SIZE: Option<core::num::NonZeroUsize> = None;

/// get the `PAGE_SIZE`
pub fn get_page_size() -> Result<usize> {
    // use the cached value if successful calls have been made
    unsafe { if let Some(res) = PAGE_SIZE { return Ok(res.get()); } }
    // acquire the value
    unsafe { set_errno(0) }
    let sz = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if sz < 0 {
        Err(unsafe { MMapError::get() })
    } else {
        unsafe { PAGE_SIZE = Some(core::num::NonZeroUsize::new_unchecked(sz as usize)) }
        Ok(sz as usize)
    }
}

/// get the minimum alignment of memory chunks
#[inline]
pub fn get_minimum_alignment() -> Result<usize> {
    get_page_size()
}

/// allocate a memory chunk with the given size and protection flags
pub unsafe fn allocate_chunk(size: usize, protection: BitFlags<Protection>) -> Result<*mut c_void> {
    if size == 0 { return Err(MMapError::InvalidArguments); }
    set_errno(0);
    let addr = wrapped_mmap(
        core::ptr::null_mut(), size,
        protection,
        MapFlags::Private | MapFlags::Anonymous,
        INVALID_FILE_DESCRIPTOR, 0);
    if addr == libc::MAP_FAILED {
        Err(MMapError::get())
    } else {
        Ok(addr)
    }
}

/// deallocate a memory chunk
pub unsafe fn deallocate_chunk(addr: *mut c_void, size: usize) -> Result<()> {
    set_errno(0);
    if libc::munmap(addr, size) < 0 {
        Err(MMapError::get())
    } else {
        Ok(())
    }
}

fn is_power_of_2(x: usize) -> bool {
    (x - 1) & x == 0
}

/// allocate an aligned memory chunk with the given alignment, size and protection flags
pub unsafe fn aligned_allocate_chunk(
    alignment: usize, size: usize, protection: BitFlags<Protection>) -> Result<*mut c_void> {
    assert!(is_power_of_2(alignment));
    assert_eq!(size % alignment, 0);
    let alignment_mask = alignment - 1;
    let res = allocate_chunk(size + alignment, protection)?;
    let back_padding = res as usize & alignment_mask;
    let front_padding = alignment - back_padding;
    deallocate_chunk(res, front_padding)?;
    let start_addr = res.offset(front_padding as isize);
    if back_padding > 0 {
        deallocate_chunk(start_addr, back_padding)?;
    }
    Ok(start_addr)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::is_power_of_2;

    #[test]
    fn test_is_power_of_2() {
        assert!(is_power_of_2(1));
        assert!(is_power_of_2(2));
        assert!(is_power_of_2(256));
        assert!(!is_power_of_2(257));
    }
}
