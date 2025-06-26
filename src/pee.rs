use std::{
    env,
    ffi::OsString,
    io::{self, BufRead, Write},
    process::{self, Command, Stdio},
    thread,
};

use signal_hook::{consts::SIGPIPE, iterator::Signals};

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

    if !ignore_sigpipe {
        let mut signals = Signals::new([SIGPIPE])?;
        thread::spawn(move || {
            if let Some(sig) = signals.forever().next() {
                process::exit(128 + sig);
            }
        });
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
    let mut stdin = stdin.lock();
    loop {
        let buffer = stdin.fill_buf()?;
        let buflen = buffer.len();
        if buflen == 0 {
            break;
        }
        let mut peed = false;
        children.iter().for_each(|child| {
            if child
                .stdin
                .as_ref()
                .map(|mut s| s.write_all(buffer))
                .is_some()
            {
                peed = true;
            }
        });
        stdin.consume(buflen);
        if !peed {
            break;
        }
    }

    usage();
    Ok(())
}
