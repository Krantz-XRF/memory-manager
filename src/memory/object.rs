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
#[allow(dead_code)]
pub struct ObjectDescriptor {
    unpacked_field_count: usize,
    pointer_count: usize,
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

/// visit an object at some memory address.
#[allow(dead_code)]
pub unsafe fn visit_object(
    mem: &mut utils::Memory,
    handle_ctor: impl FnOnce(&mut ObjectDescriptor),
    handle_unpacked: impl FnOnce(&mut [usize]),
    mut handle_pointer: impl FnMut(&mut usize)) {
    // descriptor
    let desc = utils::assert_aligned::<ObjectDescriptor>(*mem).as_mut().unwrap();
    // unpacked fields
    let unpacked = utils::consume_mem(mem, desc.unpacked_field_count);
    handle_unpacked(unpacked);
    // pointer fields
    let pointers = utils::consume_mem(mem, desc.pointer_count);
    for p in pointers {
        handle_pointer(p);
    }
    // (possibly) mutate the descriptor
    handle_ctor(desc);
}
