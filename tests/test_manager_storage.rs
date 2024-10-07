use Jobs::{JManager, ManagerStorage};

// cargo test --test test_manager_storage -- --nocapture
#[test]
fn test_manager_storage() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = JManager::new();
    manager.load()?;
    manager.dump()?;
    Ok(())
}
