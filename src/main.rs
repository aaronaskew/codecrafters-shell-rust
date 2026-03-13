use codecrafters_shell::{parser::parser, shell_helper::ShellHelper};
use rustyline::{Config, history::DefaultHistory};

fn main() -> anyhow::Result<()> {
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut rl = rustyline::Editor::<ShellHelper, DefaultHistory>::with_config(config)?;

    let helper = ShellHelper {};

    rl.set_helper(Some(helper));

    'main_loop: loop {
        let input = match rl.readline("$ ") {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Eof) => {
                break Ok(());
            }
            Err(err) => {
                return Err(err.into());
            }
        };

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let (_, commands) = nom::Parser::parse(&mut parser, input).expect("should parse");

        // dbg!(&command);

        for command in commands {
            match command.run() {
                Ok(should_exit) => {
                    if should_exit {
                        break 'main_loop Ok(());
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }
}
