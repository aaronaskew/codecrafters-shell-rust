use anyhow::Result;
use assert_cmd::{pkg_name, prelude::*};
use assert_fs::TempDir;
use std::process::Command;

#[test]
fn ensure_fresh_build() -> Result<()> {
    let mut cmd = Command::cargo_bin(pkg_name!())?;

    cmd.assert().success();

    Ok(())
}

#[test]
fn cd() -> Result<()> {
    let temp_dir = TempDir::new()?;

    Ok(())
}
