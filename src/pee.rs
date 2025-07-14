use signal_hook::{consts::SIGPIPE, iterator::Signals};
use std::{
    env,
    ffi::OsString,
    io::{self, BufRead, Write},
    process::{self, Child, Command, Stdio},
    thread,
};

pub fn pee() -> io::Result<()> {
    let mut double_dash = false;
    let mut ignore_sigpipe = true;
    let mut ignore_write_errors = true;
    let mut commands: Vec<OsString> = Vec::new();

    for arg in env::args_os().skip(1) {
        if double_dash {
            commands.push(arg);
            continue;
        }
        match arg.to_str() {
            Some("--") => double_dash = true,
            Some("--ignore-sigpipe") => ignore_sigpipe = true,
            Some("--no-ignore-sigpipe") => ignore_sigpipe = false,
            Some("--ignore-write-errors") => ignore_write_errors = true,
            Some("--no-ignore-write-errors") => ignore_write_errors = false,
            _ => commands.push(arg),
        }
    }

    if !ignore_sigpipe {
        let mut signals = Signals::new([SIGPIPE])?;
        thread::Builder::new()
            .name(String::from("SIGPIPE-handler"))
            .spawn(move || {
                if let Some(sig) = signals.forever().next() {
                    process::exit(128 + sig);
                }
            })?;
    }

    let mut children: Vec<Child> = commands
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
                if write_result.is_err() && !ignore_write_errors {
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
