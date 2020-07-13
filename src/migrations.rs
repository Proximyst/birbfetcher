use strum_macros::EnumIter;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, EnumIter)]
pub enum Migrations {
    // Ensure this is sorted! This is required for proper migrations due to the
    // enum iterator!
    //
    // Simply add a new variant called `V2` after `V1`; no `= 2` is required.
    V1 = 1,
    V2,
}

impl Migrations {
    pub fn queries(self) -> Vec<String> {
        match self {
            Self::V1 => include_str!("migrations/0001-create-tables.sql"),
            Self::V2 => include_str!("migrations/0002-add-verified-column.sql"),
        }
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(Into::into)
        .collect()
    }
}
