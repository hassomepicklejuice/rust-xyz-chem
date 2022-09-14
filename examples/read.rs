use std::{convert::TryFrom, fs, io::BufReader};

use rust_xyz_chem::{read, File};

fn main() {
    let file = read("path/to/file.xyz").unwrap();
    println!("{file}");

    // or

    let reader = BufReader::new(fs::File::open("path/to/file.xyz").unwrap());
    let file = File::try_from(reader).unwrap();
    println!("{file}");
}
