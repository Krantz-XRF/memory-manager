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

//! memory block
use super::common;
use super::object;
use core::marker;
use common::KiB;

/// Memory block: collection of objects.
///
/// # Block Layout
///
/// ```text
/// +----------+----------+-----+----------+--------------+
/// | reserved | object 0 | ... | object N | not used yet |
/// +----------+----------+-----+----------+--------------+
///            ^                           ^
///            |                           |
///          start                        free
/// ```
///
/// - Reserved space: we may reserved some space to avoid `malloc`, etc.
/// - object 1 ~ N: objects managed by this `memory-manager`.
/// - not used yet: for future allocation, or wasted due to fragmentation.
#[derive(Copy, Clone)]
pub struct BlockDescriptor<'a> {
    /// The starting address for this block.
    /// **Invariant**: at `start` there is a valid `ObjectDescriptor`.
    pub start: *mut u8,
    /// The first free address in this block.
    /// **Invariant**: no pointers in the same block is after `free`.
    pub free: *mut u8,
    phantom: marker::PhantomData<&'a ()>,
}

/// Iterator for `Object`s.
pub struct ObjectIterator<'a> {
    current: object::Object<'a>,
    boundary: common::Address<'a>,
}

impl<'a> Iterator for ObjectIterator<'a> {
    type Item = object::Object<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let this_addr = self.current.start_address();
        let this_size = self.current.total_size();
        let next_addr = unsafe { this_addr.offset(this_size as isize) };
        if next_addr >= self.boundary { return None; }
        Some(core::mem::replace(&mut self.current, object::Object::from(next_addr)))
    }
}

impl<'a> BlockDescriptor<'a> {
    /// Size of a `Block`.
    pub const SIZE: usize = 4 * KiB;

    /// Size of a `Block` in `Word`s (`usize`s).
    pub const SIZE_IN_WORDS: usize = Self::SIZE / core::mem::size_of::<usize>();

    /// Constructor for `BlockDescriptor`.
    pub fn new(start: *mut u8) -> Self {
        BlockDescriptor { start, free: start, phantom: marker::PhantomData }
    }

    /// Iterate on the objects in this block.
    pub fn objects(&self) -> ObjectIterator<'a> {
        ObjectIterator {
            current: object::Object::from(common::Address::from(self.start)),
            boundary: common::Address::from(self.free),
        }
    }
}
