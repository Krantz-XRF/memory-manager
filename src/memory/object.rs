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

//! an object is a collection of pointers
use super::super::utils;

/// object descriptors.
pub struct ObjectDescriptor {
    /// number of unpacked fields in objects described by this descriptor
    pub unpacked_field_count: usize,
    /// number of boxed fields (i.e. pointers) in objects described by this descriptor
    pub pointer_count: usize,
}

impl ObjectDescriptor {
    /// the total size occupied by this kind of object
    pub fn total_size(&self) -> usize {
        // descriptor pointer: 1 word
        // unpacked fields: 1 word/each
        // pointers: 1 word/each
        1 + self.unpacked_field_count + self.pointer_count
    }
}

/// an object
pub struct Object<'a> {
    /// the pointer to `ObjectDescriptor`
    pub descriptor: &'a mut &'a ObjectDescriptor,
    /// the unpacked fields
    pub unpacked: &'a mut [usize],
    /// the boxed fields (i.e. pointers)
    pub pointers: &'a mut [&'a Object<'a>],
}

impl<'a> Object<'a> {
    /// the total size for this object
    /// see also `ObjectDescriptor::total_size`
    pub fn total_size(&self) -> usize {
        self.descriptor.total_size()
    }

    /// the starting address of this object
    pub fn start_address(&mut self) -> utils::Address<'a> {
        utils::Address::from(self.descriptor as *mut _)
    }
}

impl<'a> From<utils::Address<'a>> for Object<'a> {
    fn from(mut address: utils::Address<'a>) -> Self {
        unsafe {
            let descriptor = utils::consume_as_ref::<&'a ObjectDescriptor>(&mut address);
            let unpacked = utils::consume_as_slice::<usize>(
                &mut address, descriptor.unpacked_field_count);
            let pointers = utils::consume_as_slice::<&'a Object>(
                &mut address, descriptor.pointer_count);
            Object { descriptor, unpacked, pointers }
        }
    }
}
