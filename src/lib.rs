/// A file is a vec of records
pub struct File(Vec<Record>);

/// A record is a single line in a `.xyz` text file
pub enum Record {
    Count(u8),
    Comment(String),
    Atom {
        label: String,
        position: Position,
    },
}

/// A position vector is an array of `f64` per dimension
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}
