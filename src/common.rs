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

//! Common memory-related utilities.
use core::ptr;
use core::mem;
use core::marker;
use core::fmt;

/// Memory address with a valid lifetime.
///
/// We need this because raw pointers does not have a lifetime attached. Note that an `Address`
/// can be constructed from a raw pointer, and the lifetime attached is ARBITRARY, so it is on the
/// caller to guarantee the correct lifetime is specified.
///
/// # Construct from raw pointer
///
/// ```
/// use memory_manager::common::Address;
/// let raw_p = 0xDEAD_BEEF as *mut ();
/// let addr = Address::from(raw_p);
/// ```
///
/// # Debug format
///
/// ```
/// # use memory_manager::common::Address;
/// # let raw_p = 0xDEAD_BEEF as *mut ();
/// # let addr = Address::from(raw_p);
/// assert_eq!(format!("{:?}", addr), "Address(0xdeadbeef)");
/// ```
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct Address<'a> {
    address: *mut u8,
    phantom: marker::PhantomData<&'a ()>,
}

impl<'a> fmt::Debug for Address<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Address").field(&self.address).finish()
    }
}

impl<'a, T> From<*mut T> for Address<'a> {
    fn from(address: *mut T) -> Self {
        Address { address: address as *mut u8, phantom: marker::PhantomData }
    }
}

impl<'a> Address<'a> {
    /// Convert an `Address` to a raw pointer of some type `T`.
    ///
    /// Note that raw pointers do not have lifetime attached, so the lifetime is dropped after
    /// converting to a raw pointer. This should not give rise to any unsafety, as long as the
    /// `Address` was constructed correctly.
    ///
    /// # Panics
    ///
    /// This function `assert!` that the memory address is properly aligned for `T`.
    /// See also [`assert_aligned`](fn.assert_aligned.html).
    ///
    /// The following use would panic:
    ///
    /// ```should_panic
    /// use memory_manager::common::Address;
    /// let addr = Address::from(0xDEAD_BEEF as *mut ());
    /// let raw_p = addr.as_ptr::<usize>();
    /// ```
    pub fn as_ptr<T>(&self) -> *mut T {
        assert_aligned(self.address)
    }

    /// Add an offset to an `Address`.
    ///
    /// This method is analogous to `*mut T::offset`.
    ///
    /// ```
    /// use memory_manager::common::Address;
    /// let addr = Address::from(0x1000 as *mut ());
    /// assert_eq!(unsafe { addr.offset(4isize) }, Address::from(0x1004 as *mut ()));
    /// ```
    pub unsafe fn offset(&self, count: isize) -> Self {
        Address::from(self.address.offset(count))
    }
}

/// Assert that some memory is properly aligned.
///
/// Given an [`Address`](struct.Address.html), check the alignment, coerce the pointer to `*mut T`.
///
/// # Panics
///
/// Panics if input is NOT properly aligned for `T`.
///
/// The following use would panic:
///
/// ```should_panic
/// use memory_manager::common::assert_aligned;
/// let raw_p = assert_aligned::<usize>(0xDEAD_BEEF as *mut u8);
/// ```
pub fn assert_aligned<T>(mem: *mut u8) -> *mut T {
    assert_eq!(mem as usize % mem::align_of::<T>(), 0);
    mem as *mut T
}

/// Consumes a memory chunk as a slice.
///
/// Construct a slice with a given length from the current [`Address`](struct.Address.html),
/// then advance it.
///
/// ```
/// use memory_manager::common::{consume_as_slice, Address};
/// let mut addr = Address::from(0x1000 as *mut u8);
/// let _ = unsafe { consume_as_slice::<usize>(&mut addr, 20) };
/// assert_eq!(
///     addr,
///     unsafe {
///         Address::from(0x1000 as *mut u8).offset(
///             core::mem::size_of::<usize>() as isize * 20)
///     }
/// );
/// ```
pub unsafe fn consume_as_slice<'a, T>(mem: &mut Address<'a>, n: usize) -> &'a mut [T] {
    let res = ptr::slice_from_raw_parts_mut(mem.as_ptr::<T>(), n);
    let bytes = mem::size_of::<T>() * n;
    *mem = mem.offset(bytes as isize);
    res.as_mut().unwrap()
}

/// Consumes a memory chunk as a reference.
///
/// Construct a reference from the current [`Address`](struct.Address.html), then advance it.
///
/// ```
/// use memory_manager::common::{consume_as_ref, Address};
/// let mut addr = Address::from(0x1000 as *mut u8);
/// let _ = unsafe { consume_as_ref::<usize>(&mut addr) };
/// assert_eq!(
///     addr,
///     unsafe {
///         Address::from(0x1000 as *mut u8).offset(
///             core::mem::size_of::<usize>() as isize)
///     }
/// );
/// ```
pub unsafe fn consume_as_ref<'a, T>(mem: &mut Address<'a>) -> &'a mut T {
    let res = mem.as_ptr::<T>();
    let bytes = mem::size_of::<T>();
    *mem = mem.offset(bytes as isize);
    res.as_mut().unwrap()
}

/// size in Bytes
pub const B: usize = 1;
/// size in Kibibytes, as defined in IEC 60027-2
#[allow(non_upper_case_globals)]
pub const KiB: usize = 1024 * B;
/// size in Mebibytes, as defined in IEC 60027-2
#[allow(non_upper_case_globals)]
pub const MiB: usize = 1024 * KiB;
/// size in Gibibytes, as defined in IEC 60027-2
#[allow(non_upper_case_globals)]
pub const GiB: usize = 1024 * MiB;
