
use std::{env, path::PathBuf};

use Jobs::{JManager, JNodeAction, ManagerAction, ManagerStorage};

// cargo test my_pc -- --nocapture

// #[ignore]
#[test]
fn my_pc1() -> Result<(), Box<dyn std::error::Error>> {
    let pth = "E:/arduino-ide_2.3.3_Windows_64bit";
    let pth = PathBuf::from(pth);

    let mut mng = JManager::new();
    let node_h = mng.locate_node(&pth)?;
    mng.update_node(&node_h)?;
    let info = mng.get_info(&node_h)?;

    assert_eq!(info.size(), 523_203_611);
    assert_eq!(info.count_dir().unwrap(), 1266);
    assert_eq!(info.count_file().unwrap(), 6358);
    Ok(())
}

#[test]
fn my_pc2() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
    let file_path = PathBuf::from(home_dir).join("example.csv");
    let pth = "E:/arduino-ide_2.3.3_Windows_64bit";
    let pth = PathBuf::from(pth);
    
    let mut mng = JManager::new();
    mng.load(&file_path)?;
    let node_h = mng.locate_node(&pth)?;
    mng.update_node(&node_h)?;
    mng.dump(&file_path)?;
    let info = mng.get_info(&node_h)?;

    assert_eq!(info.size(), 523_203_611);
    assert_eq!(info.count_dir().unwrap(), 1266);
    assert_eq!(info.count_file().unwrap(), 6358);
    Ok(())
}
