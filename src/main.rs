use std::error::Error;

use Jobs::JManager;

mod console;
mod core;

fn main() -> Result<(), Box<dyn Error>> {
    let path = "E:\\nginx-1.26.1";
    let mut manager = JManager::new();
    manager.scan(path);
    println!("{}", manager.tree(&path).unwrap());
    Ok(())
}
