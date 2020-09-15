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

//! memory allocation utilities
use super::primitives;

use enumflags2::BitFlags;

pub use primitives::Protection;
pub use primitives::MapFlags;
pub use primitives::MMapError;
pub use primitives::Result;
use crate::memory::primitives::deallocate_chunk;

/// a memory chunk
#[derive(Debug, Eq, PartialEq)]
pub struct MemoryChunk {
    data: *mut u8,
    size: usize,
}

impl MemoryChunk {
    /// allocate a memory chunk with the provided `alignment`, `size`, and `protection`
    pub fn new(alignment: usize, size: usize, protection: BitFlags<Protection>) -> Result<Self> {
        Ok(MemoryChunk {
            data: unsafe {
                primitives::aligned_allocate_chunk(
                    alignment, size, protection)? as *mut u8
            },
            size,
        })
    }

    /// pointer to the starting address of this chunk
    pub fn data(&self) -> *mut u8 { self.data }

    /// the length of this chunk
    pub fn size(&self) -> usize { self.size }

    /// converts to a `u8` slice
    pub fn as_ref(&self) -> &[u8] {
        unsafe { core::ptr::slice_from_raw_parts(self.data, self.size).as_ref().unwrap() }
    }

    /// converts to a mutable `u8` slice
    pub fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::ptr::slice_from_raw_parts_mut(self.data, self.size).as_mut().unwrap() }
    }
}

impl Drop for MemoryChunk {
    fn drop(&mut self) {
        unsafe {
            deallocate_chunk(self.data as _, self.size)
                .expect("failed to deallocate memory: ")
        }
    }
}
