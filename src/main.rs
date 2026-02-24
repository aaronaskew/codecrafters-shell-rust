#[allow(unused_imports)]
use std::io::{self, Write};

fn main() -> std::io::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut command = String::new();

        io::stdin().read_line(&mut command)?;

        let args: Vec<String> = command.split_whitespace().map(String::from).collect();

        let first = args.first().unwrap().as_str();

        match first {
            "exit" => {
                break Ok(());
            }
            "echo" => {
                println!("{}", args[1..].join(" "));
            }
            "type" => {
                let type_arg = args[1..].join(" ");

                if matches!(type_arg.as_str(), "exit" | "echo" | "type") {
                    println!("{} is a shell builtin", type_arg);
                } else {
                    println!("{}: not found", type_arg);
                }
            }
            _ => {
                println!("{}: command not found", command.trim());
            }
        }
    }
}
