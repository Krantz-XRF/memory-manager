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
use super::super::utils;
use super::size::KiB;
use super::object;
use core::marker::PhantomData;

/// size of a `Block`
pub const BLOCK_SIZE: usize = 4 * KiB;

/// size of a `Block` in `Word`s (`usize`s)
pub const BLOCK_WORDS: usize = BLOCK_SIZE / core::mem::size_of::<usize>();

/// memory block: collection of objects
#[derive(Copy, Clone)]
pub struct BlockDescriptor<'a> {
    /// the starting address for this block
    pub start: *mut u8,
    /// the first free address in this block
    pub free: *mut u8,
    phantom: PhantomData<&'a ()>,
}

/// iterator for `Object`s
pub struct ObjectIterator<'a> {
    current: object::Object<'a>,
    boundary: utils::Address<'a>,
}

impl<'a> Iterator for ObjectIterator<'a> {
    type Item = object::Object<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let this_addr = self.current.start_address();
        let this_size = self.current.total_size();
        let next_addr = unsafe { this_addr.offset(this_size as isize) };
        if next_addr >= self.boundary { return None; }
        let mut res = object::Object::from(next_addr);
        core::mem::swap(&mut self.current, &mut res);
        Some(res)
    }
}

impl<'a> BlockDescriptor<'a> {
    /// iterate on the objects in this block
    pub fn objects(&self) -> ObjectIterator<'a> {
        ObjectIterator {
            current: object::Object::from(utils::Address::from(self.start)),
            boundary: utils::Address::from(self.free),
        }
    }
}
