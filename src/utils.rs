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

/// Memory address.
pub type Memory = *mut u8;

/// Assert that some memory is properly aligned.
pub fn assert_aligned<T>(mem: Memory) -> *mut T {
    assert_eq!(mem as usize % mem::align_of::<T>(), 0);
    mem as *mut T
}

/// Consumes a memory chunk.
pub unsafe fn consume_mem<T>(mem: &mut Memory, n: usize) -> &mut [T] {
    let res = ptr::slice_from_raw_parts_mut(assert_aligned(*mem), n);
    let bytes = mem::size_of::<T>() * n;
    *mem = (*mem).offset(bytes as isize);
    res.as_mut().unwrap()
}
