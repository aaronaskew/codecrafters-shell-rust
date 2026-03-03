use std::env;
use std::path::Path;

use is_executable::IsExecutable;
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

        let builtins = ["echo", "type", "cd", "pwd", "exit"];

        let mut candidates: Vec<Self::Candidate> = builtins
            .iter()
            .filter(|builtin| builtin.starts_with(word))
            .map(|s| s.to_string())
            .collect();

        let mut executables = find_executables_in_path(word);

        candidates.append(&mut executables);

        candidates.sort();
        candidates.dedup();

        let candidates = if candidates.len() == 1 {
            vec![format!("{} ", candidates[0])]
        } else {
            candidates
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
