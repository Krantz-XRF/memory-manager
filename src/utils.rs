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

//! common memory-related utilities
use core::ptr;
use core::mem;
use core::marker;
use core::fmt;

/// Memory address.
#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct Address<'a> {
    address: *mut u8,
    phantom: marker::PhantomData<&'a ()>,
}

impl<'a> fmt::Debug for Address<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Address {{ {:?} }}", self.address)
    }
}

impl<'a, T> From<*mut T> for Address<'a> {
    fn from(address: *mut T) -> Self {
        Address { address: address as *mut u8, phantom: marker::PhantomData }
    }
}

impl<'a> Address<'a> {
    /// convert a `Memory` to a raw pointer
    pub unsafe fn as_ptr<T>(&self) -> *mut T {
        assert_aligned(self.address)
    }

    /// add an offset to a `Memory` address
    pub unsafe fn offset(&self, count: isize) -> Self {
        Address::from(self.address.offset(count))
    }
}

/// Assert that some memory is properly aligned.
pub fn assert_aligned<T>(mem: *mut u8) -> *mut T {
    assert_eq!(mem as usize % mem::align_of::<T>(), 0);
    mem as *mut T
}

/// Consumes a memory chunk as a slice.
pub unsafe fn consume_as_slice<'a, T>(mem: &mut Address<'a>, n: usize) -> &'a mut [T] {
    let res = ptr::slice_from_raw_parts_mut(mem.as_ptr::<T>(), n);
    let bytes = mem::size_of::<T>() * n;
    *mem = mem.offset(bytes as isize);
    res.as_mut().unwrap()
}

/// Consumes a memory chunk as a reference.
pub unsafe fn consume_as_ref<'a, T>(mem: &mut Address<'a>) -> &'a mut T {
    let res = mem.as_ptr::<T>();
    let bytes = mem::size_of::<T>();
    *mem = mem.offset(bytes as isize);
    res.as_mut().unwrap()
}
