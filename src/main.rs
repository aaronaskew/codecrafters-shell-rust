#[allow(unused_imports)]
use std::io::{self, Write};

fn main() -> std::io::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut command = String::new();

        io::stdin().read_line(&mut command)?;

        if command.trim() == "exit" {
            break Ok(());
        }

        println!("{}: command not found", command.trim());
    }
}
