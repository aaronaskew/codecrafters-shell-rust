#![allow(unused)]

use std::fs::{self, File, OpenOptions};
use std::io::{Write, stderr, stdout};
use std::{
    env::{self, current_dir, home_dir, set_current_dir},
    fmt::Error,
    path::{Path, PathBuf},
    process::{self},
};

use anyhow::Result;
use is_executable::IsExecutable;


pub mod parse;
pub mod shell_helper;


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
enum StdoutKind {
    Normal,
    Redirect(String),
    Append(String),
}

impl StdoutKind {
    fn is_normal(&self) -> bool {
        matches!(self, StdoutKind::Normal)
    }
}

#[derive(Debug, Clone)]
enum StderrKind {
    Normal,
    Redirect(String),
    Append(String),
}

impl StderrKind {
    fn is_normal(&self) -> bool {
        matches!(self, StderrKind::Normal)
    }
}

#[derive(Debug, Clone)]
enum CommandToken {
    Argument(String),
    Stdout(StdoutKind),
    Stderr(StderrKind),
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
enum BuiltinCommand {
    Echo(Vec<String>),
    Type(Vec<String>),
    Cd(Vec<String>),
    Pwd,
    Exit,
}

#[derive(Debug)]
enum CommandKind {
    Builtin(BuiltinCommand),
    External(Vec<String>),
}

#[derive(Debug)]
pub struct Command {
    command_kind: CommandKind,
    stdout: StdoutKind,
    stderr: StderrKind,
}

impl Command {
    fn new(tokens: Vec<CommandToken>) -> anyhow::Result<Self> {
        let stdout: StdoutKind = if let Some(stdout_token) = tokens
            .iter()
            .find(|arg| matches!(**arg, CommandToken::Stdout(_)))
            && let CommandToken::Stdout(stdout_type) = stdout_token
        {
            stdout_type.clone()
        } else {
            StdoutKind::Normal
        };

        let stderr: StderrKind = if let Some(stderr_token) = tokens
            .iter()
            .find(|arg| matches!(**arg, CommandToken::Stderr(_)))
            && let CommandToken::Stderr(stderr_type) = stderr_token
        {
            stderr_type.clone()
        } else {
            StderrKind::Normal
        };

        let args: Vec<CommandToken> = tokens
            .iter()
            .filter_map(|arg| {
                if let CommandToken::Argument(_) = arg {
                    return Some(arg.clone());
                }

                None
            })
            .collect();

        let arg_strings: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();

        let first_arg = arg_strings.first().unwrap().as_str();

        let command_kind = match first_arg {
            "echo" => CommandKind::Builtin(BuiltinCommand::Echo(arg_strings[1..].to_vec())),
            "type" => CommandKind::Builtin(BuiltinCommand::Type(arg_strings[1..].to_vec())),
            "cd" => CommandKind::Builtin(BuiltinCommand::Cd(arg_strings[1..].to_vec())),
            "pwd" => CommandKind::Builtin(BuiltinCommand::Pwd),
            "exit" => CommandKind::Builtin(BuiltinCommand::Exit),
            _ => CommandKind::External(arg_strings),
        };

        Ok(Self {
            command_kind,
            stdout,
            stderr,
        })
    }

    /// Returns `Ok(true)` if shell should exit after this iteration. Returns `Ok(false)` if shell should continue.
    pub fn run(&self) -> Result<bool> {
        let mut stdout_writer: Box<dyn Write> = if self.stdout.is_normal() {
            Box::new(stdout().lock())
        } else {
            let path = match &self.stdout {
                StdoutKind::Append(path) | StdoutKind::Redirect(path) => path,
                _ => panic!("should not be Normal"),
            };

            let path_parts = path.split('/').collect::<Vec<&str>>();
            fs::create_dir_all(path_parts[..(path_parts.len() - 1)].join("/"))?;

            if matches!(self.stdout, StdoutKind::Redirect(_)) {
                Box::new(
                    OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(path)?,
                )
            } else {
                Box::new(
                    OpenOptions::new()
                        .append(true)
                        .truncate(false)
                        .create(true)
                        .open(path)?,
                )
            }
        };

        let mut stderr_writer: Box<dyn Write> = if self.stderr.is_normal() {
            Box::new(stderr().lock())
        } else {
            let path = match &self.stderr {
                StderrKind::Append(path) | StderrKind::Redirect(path) => path,
                _ => panic!("should not be Normal"),
            };

            let path_parts = path.split('/').collect::<Vec<&str>>();
            fs::create_dir_all(path_parts[..(path_parts.len() - 1)].join("/"))?;

            if matches!(self.stderr, StderrKind::Redirect(_)) {
                Box::new(
                    OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(path)?,
                )
            } else {
                Box::new(
                    OpenOptions::new()
                        .append(true)
                        .truncate(false)
                        .create(true)
                        .open(path)?,
                )
            }
        };

        match &self.command_kind {
            CommandKind::Builtin(builtin_command) => match builtin_command {
                BuiltinCommand::Echo(arg_strings) => {
                    writeln!(stdout_writer, "{}", arg_strings.join(" "));
                }
                BuiltinCommand::Type(arg_strings) => {
                    let type_command = arg_strings[0].clone();

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
                BuiltinCommand::Cd(arg_strings) => {
                    let path = if arg_strings[0] == "~" {
                        home_dir().unwrap()
                    } else {
                        PathBuf::from(&arg_strings[0])
                    };

                    if let Ok(exists) = path.try_exists()
                        && exists
                    {
                        set_current_dir(path)?;
                    } else {
                        writeln!(
                            stdout_writer,
                            "cd: {}: No such file or directory",
                            &arg_strings[0]
                        );
                    }
                }
                BuiltinCommand::Pwd => {
                    writeln!(stdout_writer, "{}", current_dir()?.display());
                }
                BuiltinCommand::Exit => {
                    return Ok(true);
                }
            },
            CommandKind::External(arg_strings) => {
                let first_arg = arg_strings.first().unwrap().as_str();

                if executable(first_arg).is_some() {
                    let mut command = process::Command::new(first_arg);

                    command.args(arg_strings[1..].iter());

                    let output = command.output()?;

                    let s = String::from_utf8_lossy(&output.stdout);
                    if !s.is_empty() {
                        write!(stdout_writer, "{s}");
                    }

                    let err = String::from_utf8_lossy(&output.stderr);
                    if !err.is_empty() {
                        write!(stderr_writer, "{err}");
                    }
                } else {
                    writeln!(
                        stderr_writer,
                        "{}: command not found",
                        arg_strings.join(" ")
                    );
                }
            }
        }

        Ok(false)
    }
}
