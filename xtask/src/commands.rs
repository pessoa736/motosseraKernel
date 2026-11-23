use std::{process::Command};
use crate::utils::run;


pub fn sh(c: &str, dir: &str)
{
    run(
        Command::new("sh")
        .args(["-c", c])
        .current_dir(dir)
    )
}

pub fn git(args: &str, dir: &str)
{
    sh(format!("git {args}").as_str(), dir);
}

pub fn cargo(args: &str, dir: &str)
{
    sh(format!("cargo {args}").as_str(), dir);
}