#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env::{self, current_dir, home_dir, set_current_dir},
    path::{Path, PathBuf},
    process::Command,
};

use is_executable::IsExecutable;

fn main() -> std::io::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut command = String::new();

        io::stdin().read_line(&mut command)?;

        let args: Vec<String> = command.split_whitespace().map(String::from).collect();

        let first_arg = args.first().unwrap().as_str();

        match first_arg {
            "exit" => {
                break Ok(());
            }
            "echo" => {
                println!("{}", args[1..].join(" "));
            }
            "type" => {
                let command = args[1].clone();

                if matches!(command.as_str(), "exit" | "echo" | "type" | "pwd" | "cd") {
                    println!("{} is a shell builtin", command);
                } else {
                    if let Some(executable_path) = executable(&command) {
                        println!("{} is {}", &command, executable_path.display());
                    } else {
                        println!("{}: not found", command);
                    }
                }
            }
            "pwd" => {
                println!("{}", current_dir()?.display());
            }
            "cd" => {
                let path = if args[1] == "~" {
                    home_dir().unwrap()
                } else {
                    PathBuf::from(&args[1])
                };

                if let Ok(exists) = path.try_exists()
                    && exists
                {
                    set_current_dir(path).unwrap();
                } else {
                    println!("cd: {}: No such file or directory", &args[1]);
                }
            }
            _ => {
                if executable(first_arg).is_some() {
                    let output = Command::new(first_arg).args(args[1..].iter()).output()?;

                    if output.status.success() {
                        let s = String::from_utf8_lossy(&output.stdout);
                        print!("{}", s);
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr);
                        eprintln!("Error: {}", err);
                    }
                } else {
                    println!("{}: command not found", command.trim());
                }
            }
        }
    }
}

fn executable(name: &str) -> Option<PathBuf> {
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
