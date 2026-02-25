#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env,
    path::{Path, PathBuf},
};

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
                    if let Some(executable_path) = executable(args[1].clone()) {
                        println!("{} is {}", args[1], executable_path.display());
                    } else {
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

fn executable(name: String) -> Option<PathBuf> {
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            let path_string = format!("{}/{}", path.display(), name);

            let path = Path::new(&path_string);

            if path.is_executable() {
                return Some(path.into());
            }
        }
    }

    None
}
