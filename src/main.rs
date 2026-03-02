#[allow(unused_imports)]
use anyhow::Result;
use std::io::{self, Write};

use codecrafters_shell::parse::parser;
use nom::Parser;

fn main() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();

        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let (_, command) = parser.parse(input).expect("should parse");

        // dbg!(&command);

        match command.run() {
            Ok(keep_running) => {
                if !keep_running {
                    break Ok(());
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}
