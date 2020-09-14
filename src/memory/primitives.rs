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

//! memory allocation primitives
mod unix;

/// common errors from mmap
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MMapError {
    /// arguments provided to mmap is invalid
    InvalidArguments,
    /// too much memory has been locked
    TryAgain,
    /// no memory available, or
    /// maximum number of mappings exceeded, or
    /// `RLIMIT_DATA` exceeded
    NoMemory,
    /// number of pages overflows `unsigned long` (32-bit platform)
    LengthOverflow,
    /// errors not expected
    UnknownError,
    /// no error at all, NOT EXPECTED
    /// should double-check implementation if received
    NoError,
}

/// memory allocation results
pub type Result<T> = core::result::Result<T, MMapError>;

#[cfg(unix)]
use unix as detail;
#[cfg(windows)]
use windows as detail;

pub use detail::Protection;
pub use detail::MapFlags;

pub use detail::get_page_size;

pub use detail::allocate_chunk;
pub use detail::aligned_allocate_chunk;
pub use detail::deallocate_chunk;
