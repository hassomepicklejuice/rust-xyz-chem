//! A small crate for reading an writing chemical `.xyz` files.
//! For more information on the XYZ file format, visit
//! [XYZ file format - Wikipedia](https://en.wikipedia.org/wiki/XYZ_file_format).
//!
//! # Examples
//!
//! Example for reading a `.xyz` file:
//! ```rust
//! # use std::{convert::TryFrom, fs, io::BufReader};
//! # use rust_xyz_chem::{read, File};
//! let file = read("path/to/file.xyz").unwrap();
//! println!("{file}");
//!
//! // or
//!
//! let reader = BufReader::new(fs::File::open("path/to/file.xyz").unwrap());
//! let file = File::try_from(reader).unwrap();
//! println!("{file}");
//! ```

#![allow(unused)]

use std::{
    convert::TryFrom,
    error,
    fmt::Display,
    fs,
    io::{self, BufRead, BufReader, Lines},
    num,
    path::Path,
    result,
    str::FromStr,
};

#[cfg(test)]
mod tests;

type Result<T> = result::Result<T, ParseError>;

/// A wrapper for [`ParseErrorKind`] that includes information about the line where the parsing error
/// occurred.
#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    line: usize,
}

impl ParseError {
    fn new(kind: ParseErrorKind, line: usize) -> ParseError {
        ParseError { kind, line }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}", self.kind, self.line)
    }
}

/// A wrapper for the different errors that can occur during the parsing of a [`File`].
#[derive(Debug)]
pub enum ParseErrorKind {
    MissingValue,
    ParseIntError(num::ParseIntError),
    ParseFloatError(num::ParseFloatError),
    ReadError(io::Error),
}

impl Display for ParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingValue => write!(f, "Missing label and/or value"),
            Self::ParseIntError(e) => e.fmt(f),
            Self::ParseFloatError(e) => e.fmt(f),
            Self::ReadError(e) => e.fmt(f),
        }
    }
}

impl From<num::ParseIntError> for ParseErrorKind {
    fn from(err: num::ParseIntError) -> Self {
        ParseErrorKind::ParseIntError(err)
    }
}

impl From<num::ParseFloatError> for ParseErrorKind {
    fn from(err: num::ParseFloatError) -> Self {
        ParseErrorKind::ParseFloatError(err)
    }
}

impl From<io::Error> for ParseErrorKind {
    fn from(err: io::Error) -> Self {
        ParseErrorKind::ReadError(err)
    }
}

/// A Position is an collection of 3 [`f64`]s, one for each dimension.
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}\t{}", self.x, self.y, self.z)
    }
}

impl From<Position> for [f64; 3] {
    fn from(p: Position) -> Self {
        [p.x, p.y, p.z]
    }
}

impl From<Position> for Vec<f64> {
    fn from(p: Position) -> Self {
        p.into()
    }
}

/// An atom is represented by a label ('C' for carbon, 'Ne' for Neon, ...) and a [`Position`].
#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    pub label: String,
    pub position: Position,
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.label, self.position)
    }
}

impl FromStr for Atom {
    type Err = ParseErrorKind;
    fn from_str(line: &str) -> result::Result<Self, Self::Err> {
        let mut line = line.split_whitespace();

        let label = line.next().ok_or(ParseErrorKind::MissingValue)?.to_string();
        let x = line.next().ok_or(ParseErrorKind::MissingValue)?.parse()?;
        let y = line.next().ok_or(ParseErrorKind::MissingValue)?.parse()?;
        let z = line.next().ok_or(ParseErrorKind::MissingValue)?.parse()?;

        Ok(Atom {
            label,
            position: Position { x, y, z },
        })
    }
}

/// A [`Record`] is a complete dataunit of a `.xyz` file.
/// It contains the amount of atoms, a comment, and a [`Vec`] of [`Atom`]s.
#[derive(Debug)]
pub struct Record {
    pub count: usize,
    pub comment: String,
    pub atoms: Vec<Atom>,
}

impl Record {
    pub fn new(comment: &str, atoms: &[Atom]) -> Self {
        Record {
            count: atoms.len(),
            comment: comment.to_string(),
            atoms: atoms.to_vec(),
        }
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}\n{}", self.count, self.comment)?;
        for atom in &self.atoms {
            writeln!(f, "{atom}")?;
        }
        Ok(())
    }
}

impl From<&[Atom]> for Record {
    fn from(atoms: &[Atom]) -> Self {
        Record::new("", atoms)
    }
}

/// A [`File`] is a vec of [`Record`]s, a collection of dataunits.
/// There can be multiple records in a file that can for example represent different timesteps in a
/// simulation.
#[derive(Debug)]
pub struct File {
    pub records: Vec<Record>,
}

impl File {
    pub fn new() -> Self {
        File {
            records: Vec::new(),
        }
    }

    pub fn push(&mut self, record: Record) {
        self.records.push(record);
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for record in &self.records {
            writeln!(f, "{record}")?;
        }
        Ok(())
    }
}

impl TryFrom<BufReader<fs::File>> for File {
    type Error = ParseError;
    fn try_from(reader: BufReader<fs::File>) -> Result<Self> {
        enum ParseState {
            Count,
            Comment,
            Atoms,
        }

        let lines = reader.lines();
        let mut file = File::new();
        let mut record = Record::new("", &[]);
        let mut parse_state = ParseState::Count;

        for (line_nr, line) in lines.enumerate() {
            let line = line.map_err(|err| ParseError::new(err.into(), line_nr))?;
            (record, parse_state) = match parse_state {
                ParseState::Count => {
                    if line.is_empty() {
                        (record, ParseState::Count)
                    } else {
                        record.count = line.parse().map_err(|err: num::ParseIntError| {
                            ParseError::new(err.into(), line_nr)
                        })?;
                        (record, ParseState::Comment)
                    }
                }
                ParseState::Comment => {
                    record.comment = line;
                    (record, ParseState::Atoms)
                }
                ParseState::Atoms => {
                    record
                        .atoms
                        .push(line.parse().map_err(|err| ParseError::new(err, line_nr))?);
                    if record.atoms.len() < record.count {
                        (record, ParseState::Atoms)
                    } else {
                        file.push(record);
                        (Record::new("", &[]), ParseState::Count)
                    }
                }
            };
        }

        Ok(file)
    }
}

/// Reads a chemical `.xyz` file to the [`File`] type.
pub fn read<P: AsRef<Path>>(path: P) -> Result<File> {
    let reader =
        BufReader::new(fs::File::open(path).map_err(|err| ParseError::new(err.into(), 0))?);
    File::try_from(reader)
}
