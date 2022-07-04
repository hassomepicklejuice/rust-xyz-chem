fn main() {
    let data = rust_xyz_chem::read("Projects/rust-xyz-chem/atoms.xyz");

    match data {
        Ok(d) => println!("data: {}", d),
        Err(e) => println!("error: {}", e),
    }
}
