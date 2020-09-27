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

use common::Address;
use common::MiB;

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
    pub unsafe fn data(&self) -> Address<'_> { Address::from(self.data) }

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
                self.data as _, self.size).as_ref().unwrap()
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
            core::ptr::slice_from_raw_parts_mut(
                self.data as _, self.size).as_mut().unwrap()
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

/// Mega-blocks: allocation units, we reserve `Block`s from `MegaBlock`s.
///
/// Mega-blocks are managed in a global doubly-linked list.
pub struct MegaBlock {
    /// The previous mega-block in the global list.
    pub previous: *mut MegaBlock,
    /// The next mega-block in the global list.
    pub next: *mut MegaBlock,
    /// The allocated memory chunk for this mega-block.
    pub chunk: MemoryChunk,
}

impl MegaBlock {
    /// Size of a `MegaBlock`.
    pub const SIZE: usize = 4 * MiB;

    /// Size of a `MegaBlock` in `Word`s (`usize`s).
    pub const SIZE_IN_WORDS: usize = Self::SIZE / core::mem::size_of::<usize>();

    /// Constructor for `MegaBlock`.
    pub fn new(protection: BitFlags<Protection>) -> Result<Self> {
        Ok(MegaBlock {
            previous: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
            chunk: MemoryChunk::new(Self::SIZE, Self::SIZE, protection)?,
        })
    }
}

/// Mutable iterator for mega-blocks.
pub struct MegaBlockIteratorMut<'a>(&'a mut MegaBlock);

impl<'a> Iterator for MegaBlockIteratorMut<'a> {
    type Item = &'a mut MegaBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next;
        let mut current = MegaBlockIteratorMut(unsafe { next.as_mut()? });
        core::mem::swap(self, &mut current);
        Some(current.0)
    }
}

impl<'a> From<&'a mut MegaBlock> for MegaBlockIteratorMut<'a> {
    fn from(x: &'a mut MegaBlock) -> Self {
        MegaBlockIteratorMut(x)
    }
}

/// Const iterator for mega-blocks.
pub struct MegaBlockIterator<'a>(&'a MegaBlock);

impl<'a> Iterator for MegaBlockIterator<'a> {
    type Item = &'a MegaBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next;
        let mut current = MegaBlockIterator(unsafe { next.as_ref()? });
        core::mem::swap(self, &mut current);
        Some(current.0)
    }
}

impl<'a> From<&'a MegaBlock> for MegaBlockIterator<'a> {
    fn from(x: &'a MegaBlock) -> Self {
        MegaBlockIterator(x)
    }
}
