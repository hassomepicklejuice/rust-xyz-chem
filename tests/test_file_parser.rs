use rust_xyz_chem::File;
use std::{fs, io};

#[test]
fn test_file_parser1() {
    let f = fs::File::open("path/to/file.xyz").unwrap();
    let r = io::BufReader::new(f);
    let file: File = r.try_into().unwrap();
    println!("{file}");
}
