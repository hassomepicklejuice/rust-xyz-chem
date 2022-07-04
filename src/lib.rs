mod data;
mod error;

use std::fs;
use std::path::Path;

type AtomCount = usize;
type AtomPosition = [f64; 3];

/// Reads a `.xyz` file.
pub fn read<P: AsRef<Path>>(path: P) -> error::Result<data::File> {
    let contents = fs::read_to_string(path)?;
    let mut lines = contents.lines();
    let mut line = lines.next();
    let mut line_count = 1;

    let mut file = data::File {
        data: vec![],
    };

    while line.is_some() {
        let count: AtomCount = match line.ok_or(error::FileParseError::InvalidAtomCount(line_count))?.parse() {
            Ok(n) => n,
            Err(_) => return Err(error::FileParseError::InvalidAtomCount(line_count)),
        };

        line = lines.next();
        line_count += 1;

        let comment: String = line.unwrap_or("").to_string();

        let mut data = data::Data {
            count,
            comment,
            atoms: vec![],
        };

        for _ in 0..count {
            line = lines.next();
            line_count += 1;

            let mut data_line = match line {
                Some(s) => s.split_whitespace(),
                None => return Err(error::FileParseError::EmptyLine(line_count)),
            };

            data.atoms.push(data::AtomData {
                symbol: data_line.next().ok_or(error::FileParseError::NoAtomSymbol(line_count))?.to_string(),
                position: splitwhitespace_to_position(&mut data_line, &line_count)?,
            });
        }

        file.data.push(data);

        line = lines.next();
        line_count += 1;

        match line {
            Some(s) => if !s.is_empty() {
                return Err(error::FileParseError::UnexpectedData(line_count));
            }
            None => {
                break
            }
        }
        line = lines.next();
        line_count += 1;
    }

    Ok(file)
}

fn splitwhitespace_to_position(data: &mut std::str::SplitWhitespace, line: &usize) -> error::Result<AtomPosition> {
    Ok([
        match data.next().ok_or(error::FileParseError::NoPositionData(*line))?.parse() {
            Ok(n) => n,
            Err(_) => return Err(error::FileParseError::InvalidPositionData(*line)),
        },
        match data.next().ok_or(error::FileParseError::NoPositionData(*line))?.parse() {
            Ok(n) => n,
            Err(_) => return Err(error::FileParseError::InvalidPositionData(*line)),
        },
        match data.next().ok_or(error::FileParseError::NoPositionData(*line))?.parse() {
            Ok(n) => n,
            Err(_) => return Err(error::FileParseError::InvalidPositionData(*line)),
        },
    ])
}

pub fn write<P: AsRef<Path>>(path: P, file: data::File) -> std::io::Result<()> {
    let mut contents = String::new();

    for data in file.data {
        contents.push_str(&format!("{}", data));
    }

    fs::write(path, contents)
}
