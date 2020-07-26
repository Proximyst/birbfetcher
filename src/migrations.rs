// birbfetcher - Collect bird images with ease.
// Copyright (C) 2020 Mariell Hoversholm
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, EnumIter)]
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
    pub fn queries(self) -> Vec<String> {
        match self {
            Self::V1 => include_str!("migrations/0001-create-tables.sql"),
            Self::V2 => include_str!("migrations/0002-add-verified-column.sql"),
            Self::V3 => include_str!("migrations/0003-unsigned-id-column.sql"),
        }
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(Into::into)
        .collect()
    }
}
