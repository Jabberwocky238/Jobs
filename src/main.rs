use core::DirInfo;
use std::{error::Error, io::Write, path::Path};

mod console;
mod core;
mod serialize;

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("E:\\1-code\\JS\\jw238.github.io");
    let mut dir = DirInfo::new(path);
    dir.scan();
    println!("{}", dir.tree(0, -1));
    println!("{} files", dir.count_file);
    Ok(())
}
