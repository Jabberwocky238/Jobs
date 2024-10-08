#![cfg(feature = "console")]

// use std::path::PathBuf;

use Jobs::{Console, JNodeAction, ManagerAction};

// cargo test --test test_console -- --nocapture
#[test]
fn test_console2() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();
    console.exec("cd \"E:\\QQ\\resources\\app\\versions\\9.9.7-21804\\avsdk\"")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur)?;
    assert_eq!(info.count_file().unwrap(), 59);
    assert_eq!(info.count_dir().unwrap(), 2);
    Ok(())
}

#[test]
fn test_console3() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();
    console.exec("cd \"E:\\QQ\\resources\"")?;
    // console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur)?;
    assert_eq!(info.size(), 366_390_081); // 366.39MB
    assert_eq!(info.count_file().unwrap(), 315);
    assert_eq!(info.count_dir().unwrap(), 18);

    println!("----------------------");
    console.exec("cd ..")?;
    // console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur)?;
    assert_eq!(info.size(), 628_816_819); // 628.82MB
    assert_eq!(info.count_file().unwrap(), 432);
    assert_eq!(info.count_dir().unwrap(), 35);
    Ok(())
}

#[test]
fn test_console4() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();
    console.exec("cd \"E:\\QQ\\resources\\app\\versions\\9.9.7-21804\\avsdk\"")?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur)?;
    assert_eq!(info.count_file().unwrap(), 59);
    assert_eq!(info.count_dir().unwrap(), 2);
    Ok(())
}