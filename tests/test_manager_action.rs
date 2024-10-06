use std::path::Path;

use Jobs::{JManager, ManagerAction};

// cargo test --test test_manager_action -- --nocapture
#[test]
fn test_manager_action() -> Result<(), Box<dyn std::error::Error>> {
    let pathbuf = Path::new("E:/QQ/obj/HummerSetupDll").to_path_buf();
    let mut manager = JManager::new();
    manager.create_node(&pathbuf)?;
    dbg!(manager);

    Ok(())
}
