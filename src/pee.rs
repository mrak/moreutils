use signal_hook::{consts::SIGPIPE, iterator::Signals};
use std::{
    ffi::OsString,
    io::{self, BufRead, Write},
    process::{self, Child, Command, Stdio},
    thread,
};

fn usage() {
    eprintln!(r#"Usage: pee [--[no-]ignore-sigpipe] [--[no-]ignore-write-errors] ["command"...]"#);
}

struct Args {
    ignore_sigpipe: bool,
    ignore_write_errors: bool,
    commands: Vec<OsString>,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;
    let mut ignore_sigpipe = true;
    let mut ignore_write_errors = true;
    let mut commands: Vec<OsString> = Vec::new();
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("ignore-sigpipe") => ignore_sigpipe = true,
            Long("no-ignore-sigpipe") => ignore_sigpipe = false,
            Long("ignore-write-errors") => ignore_write_errors = true,
            Long("no-ignore-write-errors") => ignore_write_errors = false,
            Value(val) => commands.push(val),
            _ => return Err(arg.unexpected()),
        }
    }
    if commands.is_empty() {
        return Err(lexopt::Error::from("expected COMMAND"));
    }

    Ok(Args {
        ignore_sigpipe,
        ignore_write_errors,
        commands,
    })
}

pub fn pee() -> io::Result<()> {
    let args: Args = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        process::exit(1);
    });

    if !args.ignore_sigpipe {
        let mut signals = Signals::new([SIGPIPE])?;
        thread::Builder::new()
            .name(String::from("SIGPIPE-handler"))
            .spawn(move || {
                if let Some(sig) = signals.forever().next() {
                    process::exit(128 + sig);
                }
            })?;
    }

    let mut children: Vec<Child> = args
        .commands
        .into_iter()
        .map(|command| {
            Command::new("/bin/sh")
                .arg("-c")
                .arg(&command)
                .stdin(Stdio::piped())
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .spawn()
                .unwrap_or_else(|_| panic!("failed to spawn \"{command:?}\""))
        })
        .collect();

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    loop {
        let buffer = stdin.fill_buf()?;
        let buflen = buffer.len();
        if buflen == 0 {
            break;
        }
        let mut peed = false;
        for child in &mut children {
            if let Some(write_result) = child.stdin.as_ref().map(|mut s| s.write_all(buffer)) {
                if write_result.is_err() && !args.ignore_write_errors {
                    exit_children(&mut children, true)?;
                    process::exit(1);
                }
                if write_result.is_ok() {
                    peed = true;
                }
            }
        }
        stdin.consume(buflen);
        if !peed {
            break;
        }
    }

    exit_children(&mut children, false)?;

    Ok(())
}

fn exit_children(children: &mut Vec<Child>, kill: bool) -> io::Result<()> {
    if kill {
        for child in &mut *children {
            let _ = child.kill();
        }
    }
    for child in children {
        let _ = child.wait();
    }
    Ok(())
}
