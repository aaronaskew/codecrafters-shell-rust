#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, path::Path};

use is_executable::IsExecutable;

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
                    let mut executable_found = false;

                    if let Some(paths) = env::var_os("PATH") {
                        for path in env::split_paths(&paths) {
                            println!("{}", path.display());

                            let path = format!("{}/{}", path.display(), type_arg);

                            if Path::new(&path).is_executable() {
                                println!("{} is {}", type_arg, path);
                                executable_found = true;
                                break;
                            }
                        }
                    }

                    if !executable_found {
                        println!("{}: not found", type_arg);
                    }
                }
            }
            _ => {
                println!("{}: command not found", command.trim());
            }
        }
    }
}
