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

//! memory allocation primitives for Windows

#![cfg(windows)]

use winapi::um::winnt::{PVOID, HANDLE};
use winapi::um::memoryapi::VirtualFree;
use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
use winapi::um::errhandlingapi::GetLastError;
use winapi::shared::basetsd::{DWORD64, SIZE_T};
use winapi::shared::minwindef::{ULONG, DWORD};
use winapi::shared::winerror::*;
use winapi::ctypes::c_void;

use super::MMapError;
use super::Result;

use enumflags2::BitFlags;

#[link(name = "mincore")]
extern "system" {
    #[no_mangle]
    fn VirtualAlloc2(
        process: HANDLE, base_address: PVOID, size: SIZE_T,
        allocation_type: ULONG, page_protection: ULONG,
        extended_parameters: PVOID, parameter_count: ULONG,
    ) -> PVOID;
}

const MEM_COMMIT: ULONG = 0x0000_1000;
const MEM_RESERVE: ULONG = 0x0000_2000;

#[allow(dead_code)]
const MEM_DECOMMIT: ULONG = 0x0000_4000;
const MEM_RELEASE: ULONG = 0x0000_8000;

#[allow(dead_code)]
const PAGE_EXECUTE: ULONG = 0x10;
#[allow(dead_code)]
const PAGE_EXECUTE_READ: ULONG = 0x20;
#[allow(dead_code)]
const PAGE_EXECUTE_READWRITE: ULONG = 0x40;
#[allow(dead_code)]
const PAGE_EXECUTE_WRITECOPY: ULONG = 0x80;
const PAGE_NOACCESS: ULONG = 0x01;
const PAGE_READ: ULONG = 0x02;
const PAGE_READWRITE: ULONG = 0x04;
#[allow(dead_code)]
const PAGE_WRITECOPY: ULONG = 0x08;

/// Memory protection flags.
///
/// These can be combined together using the `|` operator:
///
/// ```
/// use memory_manager::allocate::Protection;
/// let protection = Protection::Read | Protection::Write;
/// ```
///
/// If no access should be performed to the memory, use `Protection::NONE`:
///
/// ```
/// use memory_manager::allocate::Protection;
/// let protection = Protection::NONE;
/// ```
///
/// Note that not all combinations are supported on Windows: `Write` will always imply `Read`.
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, BitFlags)]
pub enum Protection {
    /// Pages may be read.
    Read = 1,
    /// Pages may be written.
    Write = 2,
    /// Pages may be executed.
    Exec = 4,
}

fn make_protection_flag(protection: BitFlags<Protection>) -> ULONG {
    let rw = if protection.contains(Protection::Write) {
        PAGE_READWRITE
    } else if protection.contains(Protection::Read) {
        PAGE_READ
    } else {
        PAGE_NOACCESS
    };
    if protection.contains(Protection::Exec) { rw << 4 } else { rw }
}

impl Protection {
    /// Pages may not be accessed.
    pub const NONE: BitFlags<Protection> = unsafe { core::mem::transmute(0) };
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
enum MEM_SECTION_EXTENDED_PARAMETER_TYPE {
    MemExtendedParameterInvalidType,
    MemExtendedParameterAddressRequirements,
    MemExtendedParameterNumaNode,
    MemExtendedParameterPartitionHandle,
    MemExtendedParameterUserPhysicalHandle,
    MemExtendedParameterAttributeFlags,
    MemExtendedParameterMax,
}

use MEM_SECTION_EXTENDED_PARAMETER_TYPE::*;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
struct MEM_EXTENDED_PARAMETER {
    r#type: MEM_SECTION_EXTENDED_PARAMETER_TYPE,
    _reserved: [u8; 7],
    value: MEM_EXTENDED_PARAMETER_VALUE,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
union MEM_EXTENDED_PARAMETER_VALUE {
    ulong64: DWORD64,
    pointer: PVOID,
    size: SIZE_T,
    handle: HANDLE,
    ulong: DWORD,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
struct MEM_ADDRESS_REQUIREMENTS {
    lowest_starting_address: PVOID,
    highest_ending_address: PVOID,
    alignment: SIZE_T,
}

impl MMapError {
    /// Get `MMapError` from an error code.
    pub fn from_errno(e: DWORD) -> MMapError {
        match e {
            ERROR_INVALID_PARAMETER => MMapError::InvalidArguments,
            ERROR_SUCCESS => MMapError::NoError,
            _ => MMapError::UnknownError(e),
        }
    }

    /// Get `MMapError` from the current system error code.
    pub unsafe fn get() -> MMapError {
        Self::from_errno(GetLastError())
    }
}

static mut SYS_INFO: Option<SYSTEM_INFO> = None;

fn get_sys_info() -> &'static SYSTEM_INFO {
    unsafe {
        if let Some(res) = &SYS_INFO {
            res
        } else {
            let mut info = core::mem::zeroed();
            GetSystemInfo(&mut info);
            SYS_INFO = Some(info);
            SYS_INFO.as_ref().unwrap()
        }
    }
}

/// Get the `PAGE_SIZE`.
pub fn get_page_size() -> Result<usize> {
    Ok(get_sys_info().dwPageSize as usize)
}

/// Get the minimum alignment of memory chunks.
///
/// It is generally NOT equal to `PAGE_SIZE` on Windows. Not following this alignment requirement
/// is NOT guarded by an `assert!`, but [`aligned_allocate_chunk`] will fail with `InvalidArguments`.
///
/// See also [`aligned_allocate_chunk`].
///
/// [`aligned_allocate_chunk`]: fn.aligned_allocate_chunk.html
pub fn get_minimum_alignment() -> Result<usize> {
    Ok(get_sys_info().dwAllocationGranularity as usize)
}

fn to_void_p<T>(p: &mut T) -> *mut c_void {
    p as *mut T as *mut c_void
}

/// Allocate an aligned memory chunk with the given alignment, size and protection flags.
///
/// Unlike on UNIX-like systems, aligned raw memory allocation is properly supported by an API
/// named `VirtualAlloc2`. Thus we are not manually aligning the allocated memory. This means
/// calling this function with a bad alignment will not panic, but will fail with `InvalidArguments`.
pub unsafe fn aligned_allocate_chunk(
    alignment: usize, size: usize, protection: BitFlags<Protection>) -> Result<*mut c_void> {
    let mut address_reqs: MEM_ADDRESS_REQUIREMENTS = core::mem::zeroed();
    address_reqs.alignment = alignment;
    let mut param: MEM_EXTENDED_PARAMETER = core::mem::zeroed();
    param.r#type = MemExtendedParameterAddressRequirements;
    param.value.pointer = to_void_p(&mut address_reqs);
    let mem = VirtualAlloc2(
        core::ptr::null_mut(), core::ptr::null_mut(),
        size, MEM_COMMIT | MEM_RESERVE, make_protection_flag(protection),
        to_void_p(&mut param), 1);
    if mem != core::ptr::null_mut() {
        Ok(mem)
    } else {
        Err(MMapError::get())
    }
}

/// Deallocate a memory chunk. If some memory address other than those returned by
/// `aligned_allocate_chunk` is passed to this function, it will fail with `InvalidArguments`.
pub unsafe fn deallocate_chunk(addr: *mut c_void, _size: usize) -> Result<()> {
    if 0 != VirtualFree(addr, 0, MEM_RELEASE) {
        Ok(())
    } else {
        Err(MMapError::get())
    }
}

#[cfg(test)]
mod tests {
    use super::Protection;
    use super::make_protection_flag;

    use super::PAGE_NOACCESS;
    use super::PAGE_READWRITE;
    use super::PAGE_EXECUTE_READ;
    use super::PAGE_EXECUTE_READWRITE;

    #[test]
    fn test_make_protection_flag() {
        assert_eq!(make_protection_flag(Protection::NONE), PAGE_NOACCESS);
        assert_eq!(make_protection_flag(Protection::Read | Protection::Write), PAGE_READWRITE);
        assert_eq!(make_protection_flag(Protection::Read | Protection::Exec), PAGE_EXECUTE_READ);
        assert_eq!(
            make_protection_flag(Protection::Read | Protection::Write | Protection::Exec),
            PAGE_EXECUTE_READWRITE);
    }
}
