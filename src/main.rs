use core::{DirInfo, Serializer};
use std::{error::Error, path::Path};

mod console;
mod core;

fn main() -> Result<(), Box<dyn Error>> {
    let dir = Serializer::deserialize()?;
    println!("{}", dir.tree(0, -1));
    println!("{} files", dir.count_file);
    Ok(())
}
