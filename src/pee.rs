use std::{
    env,
    ffi::OsString,
    io::{self, BufRead, BufReader, Write},
    process::{Command, Stdio},
};

fn usage() {
    eprintln!("Usage: pee [--[no-]ignore-sigpipe] [--[no-]ignore-write-errors] [[\"command\"...]]");
}

pub fn pee() -> io::Result<()> {
    let mut double_dash = false;
    let mut ignore_sigpipe = true;
    let mut ignore_write_errors = true;
    let mut commands: Vec<OsString> = Vec::new();
    let args = env::args_os().skip(1);

    for arg in args {
        if arg == "--" {
            double_dash = true;
        } else if !double_dash && arg == "--ignore-sigpipe" {
            ignore_sigpipe = true;
        } else if !double_dash && arg == "--no-ignore-sigpipe" {
            ignore_sigpipe = false;
        } else if !double_dash && arg == "--ignore-write-errors" {
            ignore_write_errors = true;
        } else if !double_dash && arg == "--no-ignore-write-errors" {
            ignore_write_errors = false;
        } else {
            commands.push(arg);
        }
    }

    let mut children = Vec::new();
    for command in commands {
        let child = Command::new("/bin/sh")
            .arg("-c")
            .arg(&command)
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()
            .unwrap_or_else(|_| panic!("failed to spawn \"{:?}\"", command));
        children.push(child);
    }

    let stdin = io::stdin();
    let stdin = BufReader::new(stdin.lock());
    let mut buffer = Vec::new();
    loop {
        let bytes_read = match stdin.read_until(b'\n', &mut buffer) {
            Ok(br) => br,
            Err(_) => break,
        };
        for child in children {
            if let Some(n) = child.stdin {
                n.write_all(&buffer);
            }
        }
    }

    usage();
    Ok(())
}
