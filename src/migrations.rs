// birbfetcher - Collect bird images with ease.
// Copyright (C) 2020-2021 Mariell Hoversholm
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use strum_macros::EnumIter;

/// Migrations from one version of data to another.
///
/// These define how to convert from old data to data we can process currently.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, EnumIter)]
#[repr(u32)] // Ensure we never get a field on a variant, and that it's always the correct size.
pub enum Migrations {
    // Ensure this is sorted! This is required for proper migrations due to the
    // enum iterator!
    //
    // Simply add a new variant called `V2` after `V1`; no `= 2` is required.
    V1 = 1,
    V2,
    V3,
}

impl Migrations {
    /// Get all queries with this migration.
    pub fn queries(self) -> Vec<String> {
        match self {
            // TODO(Proximyst): Create macro for migrations
            Self::V1 => include_str!("migrations/0001-create-tables.sql"),
            Self::V2 => include_str!("migrations/0002-add-verified-column.sql"),
            Self::V3 => include_str!("migrations/0003-unsigned-id-column.sql"),
        }
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
    }
}
