
use std::{fs::copy, process::{Command, exit}};


pub fn cp(from: &str, to: &str) 
{
    copy(from, to).unwrap_or_else(
        |e| 
        panic!("copiar {from}: {e}")
    );
}

pub fn run(cmd: &mut Command)
{
    println!("$ {cmd:?}");
    let status = cmd.status().unwrap_or_else(
        |e| 
        { 
            eprintln!("erro: {e}");
            exit(1)
        }

    );

    if !status.success() 
    {exit(status.code().unwrap_or(1));}
}
