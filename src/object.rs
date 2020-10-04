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

//! An object is effectively a collection of pointers.
use super::common;

/// Object descriptors.
///
/// # Object Layout
///
/// ```text
/// ┏━━━━━━━━━━━━━━━━━━━━━━━┯━━━━━━━━━━━━━━━━━┯━━━━━━━━━━┓
/// ┃ pointer to descriptor │ unpacked fields │ pointers ┃
/// ┗━━━━━━━━━━━━━━━━━━━━━━━┷━━━━━━━━━━━━━━━━━┷━━━━━━━━━━┛
/// ```
///
/// All unpacked fields are gathered at the front of the object. Therefore, all the pointer fields
/// are left at the back. This makes the object descriptor simple: 2 words determines the layout.
pub struct ObjectDescriptor {
    /// Number of unpacked fields in objects described by this descriptor.
    pub unpacked_field_count: usize,
    /// Number of boxed fields (i.e. pointers) in objects described by this descriptor.
    pub pointer_count: usize,
}

impl ObjectDescriptor {
    /// The total size occupied by this kind of object.
    /// Always aligned to a `Word` (i.e. `usize`).
    ///
    /// Size is calculated as follows:
    ///
    /// - Descriptor Pointer: 1 word
    /// - Unpacked Fields: 1 word/each
    /// - Pointers: 1 word/each
    pub fn total_size(&self) -> usize {
        1 + self.unpacked_field_count + self.pointer_count
    }
}

/// An object, with a lifetime attached.
pub struct Object<'a> {
    /// The pointer to `ObjectDescriptor`.
    pub descriptor: &'a mut &'a ObjectDescriptor,
    /// The unpacked fields.
    pub unpacked: &'a mut [usize],
    /// The boxed fields (i.e. pointers).
    pub pointers: &'a mut [&'a Object<'a>],
}

impl<'a> Object<'a> {
    /// The total size for this object.
    /// See also [`ObjectDescriptor::total_size`](struct.ObjectDescriptor.html#method.total_size).
    pub fn total_size(&self) -> usize {
        self.descriptor.total_size()
    }

    /// The starting address of this object, i.e. where the pointer to
    /// [`ObjectDescriptor`](struct.ObjectDescriptor.html) is stored.
    pub fn start_address(&mut self) -> common::Address<'a> {
        common::Address::from(self.descriptor as *mut _)
    }
}

impl<'a> From<common::Address<'a>> for Object<'a> {
    fn from(mut address: common::Address<'a>) -> Self {
        unsafe {
            let descriptor = common::consume_as_ref::<&'a ObjectDescriptor>(&mut address);
            let unpacked = common::consume_as_slice::<usize>(
                &mut address, descriptor.unpacked_field_count);
            let pointers = common::consume_as_slice::<&'a Object>(
                &mut address, descriptor.pointer_count);
            Object { descriptor, unpacked, pointers }
        }
    }
}
