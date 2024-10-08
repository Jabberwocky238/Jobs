#![cfg(feature = "console")]

use Jobs::{Console, JNodeAction, ManagerAction};

// cargo test --test test_console -- --nocapture

#[test]
fn test_console() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();
    console.cd("E:/QQ")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur)?;
    assert_eq!(info.size(), 628_816_819); // 628.82MB
    assert_eq!(info.count_file().unwrap(), 432);
    assert_eq!(info.count_dir().unwrap(), 35);

    println!("----------------------");
    console.cd("resources")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur)?;
    assert_eq!(info.size(), 366_390_081); // 366.39MB
    assert_eq!(info.count_file().unwrap(), 315);
    assert_eq!(info.count_dir().unwrap(), 18);
    Ok(())
}

#[test]
fn test_console2() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();
    console.cd("E:/QQ\\resources\\app\\versions\\9.9.7-21804\\avsdk")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur)?;
    assert_eq!(info.count_file().unwrap(), 59);
    assert_eq!(info.count_dir().unwrap(), 2);
    Ok(())
}