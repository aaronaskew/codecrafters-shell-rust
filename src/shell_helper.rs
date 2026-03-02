use rustyline::Helper;
use rustyline::completion::{Candidate, Completer, Pair, extract_word};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

#[derive(Debug)]
pub struct ShellHelper {}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let (start, word) = extract_word(line, pos, Some('\\'), |c| matches!(c, ' ' | '\t'));

        let is_command = line[..start].trim().is_empty();

        if is_command {
            let mut candidates = Vec::new();
            let builtins = ["echo", "type", "cd", "pwd", "exit"];

            for builtin in builtins {
                if builtin.starts_with(word) {
                    candidates.push(Self::Candidate {
                        display: builtin.to_string(),
                        replacement: format!("{} ", builtin),
                    });
                }
            }

            candidates.sort_by(|a, b| a.display.cmp(&b.display));
            candidates.dedup_by(|a, b| a.display == b.display);

            return Ok((start, candidates));
        }

        Ok((0, Vec::with_capacity(0)))
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
