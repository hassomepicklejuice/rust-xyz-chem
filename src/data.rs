type AtomCount = usize;
type AtomPosition = [f64; 3];

/// The data for each atom.
#[derive(Debug)]
pub struct AtomData {
    /// The atomic symbol used to represent the element.
    pub symbol: String,
    /// The position of the atom as cartesion coordinates in Ångström (1e-10 m).
    pub position: AtomPosition,
}

/// Represents the data in one block of a `.xyz` file.
pub struct Data {
    /// Count of atoms in the structure.
    /// This is found on the first line of the file.
    pub count: AtomCount,
    /// A comment, title, or filename.
    /// This is found on the second line of the file.
    pub comment: String,
    /// The remaining lines of the file contain information about the positions of the atoms.
    pub atoms: Vec<AtomData>,
}
impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();

        let head = format!("{}\n{}", self.count, self.comment);
        string.push_str(&head);

        for atom in &self.atoms {
            let entry = format!(
                "\n{}\t{}\t{}\t{}",
                atom.symbol, atom.position[0], atom.position[1], atom.position[2]
            );
            string.push_str(&entry);
        }
        writeln!(f, "{}", string)
    }
}

/// Represents the data in a `.xyz` file.
pub struct File {
    /// Vec of data blocks.
    pub data: Vec<Data>,
}
impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for d in &self.data {
            writeln!(f, "{}", d)?
        }
        write!(f, "")
    }
}
