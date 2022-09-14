use std::{convert::TryInto, fs, io::BufReader};

use crate::{Atom, Position};

#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn test_atom_parser1() {
    assert_eq!(
        "C\t2.2453\t4.56\t5".parse::<Atom>().unwrap(),
        Atom {
            label: String::from("C"),
            position: Position {
                x: 2.2453,
                y: 4.56,
                z: 5.,
            },
        }
    );
}

#[test]
#[should_panic]
fn test_atom_parser2() {
    "\t2.2453\t4.56\t5".parse::<Atom>().unwrap();
}

#[test]
#[should_panic]
fn test_atom_parser3() {
    "C\t2,2453\t4.56\t5".parse::<Atom>().unwrap();
}

#[test]
fn test_file_parser1() {
    let f = fs::File::open("tests/testfile.xyz").unwrap();
    let r = BufReader::new(f);
    let file: crate::File = r.try_into().unwrap();
    println!("{file}");
}
