
use std::path::PathBuf;

use Jobs::{JManager, JNodeAction, ManagerAction};

// cargo test my_pc -- --nocapture

#[test]
fn my_pc1() -> Result<(), Box<dyn std::error::Error>> {
    let mut mng = JManager::new();
    let pth = "E:/arduino-ide_2.3.3_Windows_64bit";
    let pth = PathBuf::from(pth);
    let node_h = mng.locate_node(&pth)?;
    mng.update_node(&node_h)?;
    let info = mng.get_info(&node_h)?;

    assert_eq!(info.size(), 523_203_611);
    assert_eq!(info.count_dir().unwrap(), 1266);
    assert_eq!(info.count_file().unwrap(), 6358);
    Ok(())
}

