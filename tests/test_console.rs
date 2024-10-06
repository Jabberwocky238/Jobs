use Jobs::{Console, ManagerAction};

// cargo test --test test_console -- --nocapture
#[test]
fn test_console() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();
    console.cd("E:/QQ")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur);
    assert_eq!(info.size, 628_816_819); // 628.82MB
    assert_eq!(info.count_file, 432);
    assert_eq!(info.count_dir, 35);

    println!("----------------------");
    console.cd("obj")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    let cur = console.manager.locate_node(&console.current)?;
    let info = console.manager.get_info(&cur);
    assert_eq!(info.count_dir, 13);
    Ok(())
}