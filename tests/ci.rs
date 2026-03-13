use std::process::Command;

use anyhow::Result;
use assert_cmd::pkg_name;
use assert_fs::{
    TempDir,
    prelude::{FileWriteStr, PathChild},
};
use escargot::CargoBuild;
use rand::{prelude::*, rng};
use rexpect::session::{PtyReplSession, spawn_command};

fn random_name() -> String {
    let names = ["john", "paul", "george", "ringo", "huey", "dewey", "luey"];

    let mut rng = rng();

    (*names.choose(&mut rng).unwrap()).into()
}

fn shell_session() -> Result<PtyReplSession> {
    let path = format!(
        "{}",
        CargoBuild::new()
            .bin(pkg_name!())
            .current_release()
            .run()?
            .path()
            .display()
    );

    let mut cmd = Command::new(path);
    cmd.env("TERM", "dumb");

    let mut shell = PtyReplSession {
        // for `echo_on` you need to figure that out by trial and error.
        // For bash and python repl it is false
        echo_on: false,

        // used for `wait_for_prompt()`
        prompt: "$ ".to_owned(),
        pty_session: spawn_command(cmd, Some(2000))?,
        // command which is sent when the instance of this struct is dropped
        // in the below example this is not needed, but if you don't explicitly
        // exit a REPL then rexpect tries to send a SIGTERM and depending on the repl
        // this does not end the repl and would end up in an error
        quit_command: Some("exit".to_owned()),
    };
    shell.wait_for_prompt()?;
    Ok(shell)
}

#[test]
fn cd() -> Result<()> {
    let temp_dir = TempDir::new()?;

    let mut shell = shell_session()?;
    shell.send_line(format!("cd 4 {}", temp_dir.display()).as_str())?;
    shell.wait_for_prompt()?;
    shell.send_line("exit")?;
    shell.exp_eof()?;

    Ok(())
}

#[test]
fn cat() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let file = temp_dir.child(format!("{}.txt", random_name()));
    file.write_str("test")?;

    let mut shell = shell_session()?;
    shell.send_line(format!("cat {}", file.display()).as_str())?;
    shell.exp_string("test")?;
    shell.wait_for_prompt()?;
    shell.send_line("exit")?;
    shell.exp_eof()?;

    Ok(())
}
