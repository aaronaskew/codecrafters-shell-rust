use std::env::{self, current_dir};
use std::fs::read_dir;
use std::path::Path;

use is_executable::IsExecutable;
use itertools::Itertools;
use rustyline::Helper;
use rustyline::completion::{Candidate, Completer, Pair, extract_word};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

pub fn find_executables_in_path(start: &str) -> Vec<String> {
    let mut executables = Vec::new();

    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            if let Ok(entries) = path.read_dir() {
                for entry in entries.flatten() {
                    let filename = entry.file_name().display().to_string();

                    if filename.starts_with(start)
                        && Path::new(&format!("{}/{}", path.display(), filename)).is_executable()
                    {
                        executables.push(filename.to_string());
                    }
                }
            }
        }
    }

    executables
}

#[derive(Debug)]
pub struct ShellHelper {}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let (start, word) = extract_word(line, pos, Some('\\'), |c| matches!(c, ' ' | '\t'));

        let is_command = line[..start].is_empty();

        let mut candidates = vec![];

        if is_command {
            let builtins = ["echo", "type", "cd", "pwd", "exit"];

            candidates = builtins
                .iter()
                .filter(|builtin| builtin.starts_with(word))
                .map(|s| s.to_string())
                .collect();

            let mut executables = find_executables_in_path(word);

            candidates.append(&mut executables);
        } else {
            let cwd = current_dir()?;

            let mut path_splits = word.split('/').collect::<Vec<_>>();

            // dbg!(&path_splits);

            let relative_search_path = if word.contains('/') {
                path_splits[..path_splits.len() - 1].join("/")
            } else {
                String::new()
            };

            // dbg!(&relative_search_path);

            let (absolute_search_path, partial_filename) = if !relative_search_path.is_empty() {
                let partial_filename = path_splits.pop().unwrap();

                (cwd.join(&relative_search_path), partial_filename)
            } else {
                (cwd.clone(), word)
            };

            // dbg!(&absolute_search_path, &partial_filename);

            if let Ok(entries) = absolute_search_path.read_dir() {
                for entry in entries.flatten() {
                    // dbg!(&entry);

                    if entry
                        .file_name()
                        .display()
                        .to_string()
                        .starts_with(partial_filename)
                    {
                        // dbg!(&entry);

                        let path_string = if entry.path().is_dir() {
                            // Directory
                            if relative_search_path.is_empty() {
                                format!("{}", entry.file_name().display())
                            } else {
                                format!("{}/{}", relative_search_path, entry.file_name().display())
                            }
                        } else {
                            // File
                            if absolute_search_path != cwd {
                                // if there is a relative path from cwd
                                format!("{}/{}", relative_search_path, entry.file_name().display())
                            } else {
                                entry.file_name().display().to_string()
                            }
                        };

                        candidates.push(path_string);
                    }
                }
            }
        }

        // dbg!(&candidates);

        candidates.sort();
        candidates.dedup();

        let candidates = if candidates.len() == 1 {
            if let Ok(cwd) = current_dir()
                && cwd.join(&candidates[0]).is_dir()
            {
                vec![format!("{}/", candidates[0])]
            } else {
                vec![format!("{} ", candidates[0])]
            }
        } else {
            candidates.clone()
        };

        Ok((start, candidates))
    }

    fn update(
        &self,
        line: &mut rustyline::line_buffer::LineBuffer,
        start: usize,
        elected: &str,
        cl: &mut rustyline::Changeset,
    ) {
        let end = line.pos();
        line.replace(start..end, elected, cl);
    }
}

impl Validator for ShellHelper {}

impl Highlighter for ShellHelper {}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        let _ = (line, pos, ctx);
        None
    }
}
