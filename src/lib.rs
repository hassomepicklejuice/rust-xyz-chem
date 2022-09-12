use std::{str::FromStr, num::ParseFloatError, io::{BufReader, BufRead}, path::Path, fs, fmt::Display};

/// A position vector is an array of `f64` in 3 dimensions
#[derive(Debug)]
pub struct Position {
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
#[derive(Debug)]
pub struct Atom {
    label: String,
    position: Position,
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.label, self.position)
    }
}

impl FromStr for Atom {
    type Err = ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let string: Vec<&str> = s.split_whitespace().collect();

        let label = string[0].to_string();
        let x = string[1].parse()?;
        let y = string[2].parse()?;
        let z = string[3].parse()?;

        Ok(
            Atom {
                label,
                position: Position { x, y, z }
            }
        )
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
        let mut s = format!("{}\n{}\n", self.count, self.comment);
        for atom in &self.atoms {
            s.push_str(&format!("{atom}"));
        }
        write!(f, "{s}")
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

impl TryFrom<BufReader<fs::File>> for File {
    type Error = Box<dyn std::error::Error>;
    fn try_from(r: BufReader<fs::File>) -> Result<Self, Self::Error> {

        enum ParseState {
            Count,
            Comment,
            Atoms,
        }
        use ParseState::*;

        let lines = r.lines();
        let mut file = File::new();
        let mut record = Record::new();
        let mut parse_state = Count;

        for line in lines {
            let line = line?;
            (record, parse_state) = match parse_state {
                Count => {
                    if line.is_empty() {
                        (record, Count)
                    } else {
                        record.count = line.parse()?;
                        (record, Comment)
                    }
                },
                Comment => {
                    record.comment = line;
                    (record, Atoms)
                },
                Atoms => {
                    record.atoms.push(line.parse()?);
                    if record.atoms.len() < record.count {
                        (record, Atoms)
                    } else {
                        file.push(record);
                        (Record::new(), Count)
                    }
                },
            };
        }

        Ok(file)
    }
}

/// Reads a chemical `.xyz` file
pub fn read<P: AsRef<Path>>(path: P) -> Result<File, Box<dyn std::error::Error>> {
    let reader = BufReader::new(fs::File::open(path)?);
    File::try_from(reader)
}

