use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take},
    character::complete::{char, space0, space1},
    combinator::{all_consuming, opt, value},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated},
};

use crate::*;

fn parse_unquoted_content(input: &str) -> IResult<&str, String> {
    alt((is_not(" \t\r\n\"'\\"), preceded(char('\\'), take(1usize))))
        .map(String::from)
        .parse(input)
}

fn parse_single_quoted_content(input: &str) -> IResult<&str, String> {
    delimited(tag("'"), opt(is_not("'")), tag("'"))
        .map(|s| String::from(s.unwrap_or_default()))
        .parse(input)
}

fn parse_double_quoted_content(input: &str) -> IResult<&str, String> {
    delimited(
        char('"'),
        opt(escaped_transform(
            is_not(r#""\"#),
            '\\',
            alt((value("\\", tag("\\")), value("\"", tag("\"")))),
        )),
        char('"'),
    )
    .map(|s| s.unwrap_or_default())
    .parse(input)
}

fn parse_content(input: &str) -> IResult<&str, String> {
    many1(alt((
        parse_single_quoted_content,
        parse_double_quoted_content,
        parse_unquoted_content,
    )))
    .map(|parts| parts.join(""))
    .parse(input)
}

fn parse_stdout_token(input: &str) -> IResult<&str, CommandToken> {
    preceded(
        terminated(alt((tag(">"), tag("1>"))), space0),
        parse_content,
    )
    .map(|path| CommandToken::Stdout(StdoutKind::Redirect(path)))
    .parse(input)
}

fn parse_stderr_token(input: &str) -> IResult<&str, CommandToken> {
    preceded(terminated(tag("2>"), space0), parse_content)
        .map(|path| CommandToken::Stderr(StderrKind::Redirect(path)))
        .parse(input)
}

fn parse_argument_token(input: &str) -> IResult<&str, CommandToken> {
    parse_content.map(CommandToken::Argument).parse(input)
}

fn parse_token(input: &str) -> IResult<&str, CommandToken> {
    alt((parse_stdout_token, parse_stderr_token, parse_argument_token)).parse(input)
}

pub fn parser(input: &str) -> IResult<&str, Command> {
    let input = input.trim();

    all_consuming(separated_list1(space1, parse_token))
        .map(|tokens| Command::new(tokens).unwrap())
        .parse(input)
}
