#![allow(unused)]

use std::fs::File;
use std::io::{Write, stdout};
use std::{
    env::{self, current_dir, home_dir, set_current_dir},
    fmt::Error,
    path::{Path, PathBuf},
    process::{self},
};

use anyhow::Result;
use is_executable::IsExecutable;

pub mod parse;

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

#[derive(Debug, Clone)]
enum StdoutType {
    Normal,
    Redirect(String),
    Append(String),
}

impl StdoutType {
    fn is_normal(&self) -> bool {
        matches!(self, StdoutType::Normal)
    }
}

#[derive(Debug, Clone)]
enum StderrType {
    Normal,
    Redirect(String),
    Append(String),
}

#[derive(Debug, Clone)]
enum CommandToken {
    Argument(String),
    Stdout(StdoutType),
    Stderr(StderrType),
}

impl std::fmt::Display for CommandToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandToken::Argument(arg) => write!(f, "{}", arg.clone()),
            _ => Err(Error),
        }
    }
}

#[derive(Debug)]
pub struct Command {
    args: Vec<CommandToken>,
    stdout: StdoutType,
}

impl Command {
    fn new(tokens: Vec<CommandToken>) -> Self {
        let stdout: StdoutType = if let Some(stdout_token) = tokens
            .iter()
            .find(|arg| matches!(**arg, CommandToken::Stdout(_)))
            && let CommandToken::Stdout(stdout_type) = stdout_token
        {
            stdout_type.clone()
        } else {
            StdoutType::Normal
        };

        let args = tokens
            .iter()
            .filter_map(|arg| {
                if let CommandToken::Argument(_) = arg {
                    return Some(arg.clone());
                }

                None
            })
            .collect();

        Self { args, stdout }
    }

    pub fn run(&self) -> Result<bool> {
        let mut stdout_writer: Box<dyn Write> = match &self.stdout {
            StdoutType::Normal => Box::new(stdout().lock()),
            StdoutType::Redirect(path) => Box::new(File::create(path)?),
            StdoutType::Append(_) => todo!(),
        };

        let arg_strings: Vec<String> = self.args.iter().map(|arg| arg.to_string()).collect();

        let first_arg = arg_strings.first().unwrap().as_str();

        match first_arg {
            "exit" => {
                return Ok(false);
            }
            "echo" => {
                writeln!(stdout_writer, "{}", arg_strings[1..].join(" "));
            }
            "type" => {
                let type_command = arg_strings[1].clone();

                if matches!(
                    type_command.as_str(),
                    "exit" | "echo" | "type" | "pwd" | "cd"
                ) {
                    writeln!(stdout_writer, "{} is a shell builtin", type_command);
                } else {
                    if let Some(executable_path) = executable(&type_command) {
                        writeln!(
                            stdout_writer,
                            "{} is {}",
                            &type_command,
                            executable_path.display()
                        );
                    } else {
                        writeln!(stdout_writer, "{}: not found", type_command);
                    }
                }
            }
            "pwd" => {
                writeln!(stdout_writer, "{}", current_dir()?.display());
            }
            "cd" => {
                let path = if arg_strings[1] == "~" {
                    home_dir().unwrap()
                } else {
                    PathBuf::from(&arg_strings[1])
                };

                if let Ok(exists) = path.try_exists()
                    && exists
                {
                    set_current_dir(path)?;
                } else {
                    writeln!(
                        stdout_writer,
                        "cd: {}: No such file or directory",
                        &arg_strings[1]
                    );
                }
            }
            _ => {
                if executable(first_arg).is_some() {
                    let mut command = process::Command::new(first_arg);

                    command.args(arg_strings[1..].iter());

                    match &self.stdout {
                        StdoutType::Normal => {}
                        StdoutType::Redirect(path) => {
                            command.stdout(File::create(path)?);
                        }
                        StdoutType::Append(_) => todo!(),
                    }

                    let output = command.output()?;

                    if output.status.success() {
                        let s = String::from_utf8_lossy(&output.stdout);
                        print!("{}", s);
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr);
                        eprint!("{err}");
                    }
                } else {
                    eprintln!("{}: command not found", arg_strings.join(" "));
                }
            }
        }

        Ok(true)
    }
}
