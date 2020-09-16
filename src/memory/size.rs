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

//! common size units
#![allow(non_upper_case_globals)]

/// size in Bytes
pub const B: usize = 1;
/// size in Kibibytes, as defined in IEC 60027-2
pub const KiB: usize = 1024 * B;
/// size in Mebibytes, as defined in IEC 60027-2
pub const MiB: usize = 1024 * KiB;
/// size in Gibibytes, as defined in IEC 60027-2
pub const GiB: usize = 1024 * MiB;
