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

//! Memory allocation utilities.
use super::primitives;
use super::common;

use enumflags2::BitFlags;

pub use primitives::Protection;
pub use primitives::MMapError;
pub use primitives::Result;

/// Memory chunk.
///
/// Automatically deallocates the memory when dropped.
///
/// ```
/// use memory_manager::allocate::{MemoryChunk, Protection};
/// use memory_manager::primitives::get_minimum_alignment;
/// # use memory_manager::primitives::MMapError;
/// let a = get_minimum_alignment()?;
/// let chunk = MemoryChunk::new(a, 4096, Protection::NONE)?;
/// // memory is deallocated here
/// # Ok::<(), MMapError>(())
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct MemoryChunk {
    data: *mut u8,
    size: usize,
}

impl MemoryChunk {
    /// Allocate a memory chunk with the provided `alignment`, `size`, and `protection`.
    pub fn new(alignment: usize, size: usize, protection: BitFlags<Protection>) -> Result<Self> {
        Ok(MemoryChunk {
            data: unsafe {
                primitives::aligned_allocate_chunk(
                    alignment, size, protection)? as *mut u8
            },
            size,
        })
    }

    /// Pointer to the starting address of this chunk.
    pub fn data(&self) -> *mut u8 { self.data }

    /// Length of this chunk.
    pub fn size(&self) -> usize { self.size }
}

impl<T> AsRef<[T]> for MemoryChunk {
    /// Converts to a slice of some type `T`.
    ///
    /// # Panics
    ///
    /// Panics if `data` is not properly aligned for `T`.
    fn as_ref(&self) -> &[T] {
        unsafe {
            core::ptr::slice_from_raw_parts(
                common::assert_aligned(
                    self.data), self.size).as_ref().unwrap()
        }
    }
}

impl<T> AsMut<[T]> for MemoryChunk {
    /// Converts to a mutable slice of some type `T`.
    ///
    /// # Panics
    ///
    /// Panics if `data` is not properly aligned for `T`.
    fn as_mut(&mut self) -> &mut [T] {
        unsafe {
            core::ptr::slice_from_raw_parts_mut(common::assert_aligned(
                self.data), self.size).as_mut().unwrap()
        }
    }
}

impl Drop for MemoryChunk {
    fn drop(&mut self) {
        unsafe {
            primitives::deallocate_chunk(self.data as _, self.size)
                .expect("failed to deallocate memory: ")
        }
    }
}
