use codecrafters_shell::{parser::parser, shell_helper::ShellHelper};
use rustyline::history::DefaultHistory;

fn main() -> anyhow::Result<()> {
    let mut rl = rustyline::Editor::<ShellHelper, DefaultHistory>::new()?;

    let helper = ShellHelper {};

    rl.set_helper(Some(helper));

    loop {
        let input = rl.readline("$ ")?;

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let (_, command) = nom::Parser::parse(&mut parser, input).expect("should parse");

        // dbg!(&command);

        match command.run() {
            Ok(should_exit) => {
                if should_exit {
                    break Ok(());
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}
