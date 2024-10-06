use Jobs::Console;

// cargo test --test test_console -- --nocapture
#[test]
fn test_console() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();
    console.cd("E:/QQ")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    println!("----------------------");
    console.cd("obj")?;
    console.ls()?;
    console.scan()?;
    console.show()?;
    Ok(())
}