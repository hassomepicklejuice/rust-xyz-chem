#![allow(unused)]

use std::{
    error,
    fmt::Display,
    fs,
    io::{self, BufRead, BufReader},
    num,
    path::Path,
    result,
    str::FromStr,
};

#[cfg(test)]
mod tests;

type Result<T> = result::Result<T, ParseError>;

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

#[derive(Debug)]
enum ParseErrorKind {
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

/// A position vector is an array of `f64` in 3 dimensions
#[derive(Debug, PartialEq)]
struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}\t{}", self.x, self.y, self.z)
    }
}

/// An atom is represented by a label ('C' for carbon, 'Ne' for Neon, ...) and a position
#[derive(Debug, PartialEq)]
struct Atom {
    label: String,
    position: Position,
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

/// A Record is a complete dataunit in a `.xyz` file. There can be multiple records in a file that
/// can for example represent different timesteps in a simulation.
#[derive(Debug)]
pub struct Record {
    count: usize,
    comment: String,
    atoms: Vec<Atom>,
}

impl Record {
    fn new() -> Self {
        Record {
            count: 0,
            comment: String::new(),
            atoms: Vec::new(),
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

/// A file is a vec of records
#[derive(Debug)]
pub struct File {
    records: Vec<Record>,
}

impl File {
    fn new() -> Self {
        File {
            records: Vec::new(),
        }
    }

    fn push(&mut self, record: Record) {
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
    fn try_from(reader: BufReader<fs::File>) -> result::Result<Self, Self::Error> {
        enum ParseState {
            Count,
            Comment,
            Atoms,
        }
        use ParseState::*;

        let lines = reader.lines();
        let mut file = File::new();
        let mut record = Record::new();
        let mut parse_state = Count;

        for (line_nr, line) in lines.enumerate() {
            let line = line.or_else(|err| Err(ParseError::new(err.into(), line_nr)))?;
            (record, parse_state) = match parse_state {
                Count => {
                    if line.is_empty() {
                        (record, Count)
                    } else {
                        record.count = line.parse().or_else(|err: num::ParseIntError| {
                            Err(ParseError::new(err.into(), line_nr))
                        })?;
                        (record, Comment)
                    }
                }
                Comment => {
                    record.comment = line;
                    (record, Atoms)
                }
                Atoms => {
                    record.atoms.push(
                        line.parse()
                            .or_else(|err| Err(ParseError::new(err, line_nr)))?,
                    );
                    if record.atoms.len() < record.count {
                        (record, Atoms)
                    } else {
                        file.push(record);
                        (Record::new(), Count)
                    }
                }
            };
        }

        Ok(file)
    }
}

/// Reads a chemical `.xyz` file
pub fn read<P: AsRef<Path>>(path: P) -> Result<File> {
    let reader =
        BufReader::new(fs::File::open(path).or_else(|err| Err(ParseError::new(err.into(), 0)))?);
    File::try_from(reader)
}
