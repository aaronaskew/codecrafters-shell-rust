#[allow(unused_imports)]
use std::io::{self, Write};

fn main() -> std::io::Result<()> {
    print!("$ ");
    io::stdout().flush()?;

    let mut command = String::new();

    io::stdin().read_line(&mut command)?;

    println!("{}: command not found", command.trim());

    Ok(())
}
