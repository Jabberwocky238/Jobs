use std::error::Error;
use Jobs::run;

fn main() -> Result<(), Box<dyn Error>> {
    run()?;
    Ok(())
}
