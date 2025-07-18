use std::ffi::OsString;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::process::exit;

fn usage() {
    eprintln!("Usage: ifne [-n] command");
}

struct Args {
    invert: bool,
    command: OsString,
    arguments: Vec<OsString>,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;
    let mut invert = false;
    let mut command: Option<OsString> = None;
    let mut arguments: Vec<OsString> = Vec::new();
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('n') if command.is_none() => invert = true,
            Value(cmd) if command.is_none() => command = Some(cmd),
            Value(arg) => arguments.push(arg),
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        invert,
        command: command.ok_or("missing argument COMMAND")?,
        arguments,
    })
}

pub fn ifne() -> io::Result<()> {
    let args: Args = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        exit(1);
    });

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut peek_buffer = [0; 1];
    let r = stdin.read(&mut peek_buffer);
    let has_content = matches!(r, Ok(1));

    if args.invert {
        // -n was passed.
        // if stdin is empty, execute program
        // if stdin is not empty, pass through to stdout
        if has_content {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(&peek_buffer)?;
            io::copy(&mut stdin, &mut stdout)?;
        } else {
            Command::new(args.command)
                .args(args.arguments)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
        }
    } else if has_content {
        // -n was NOT passed.
        // if stdin is not empty, execute program with stdin content
        // if stdin is empty, do nothing
        let mut cmd = Command::new(args.command)
            .args(args.arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut cmdstdin = cmd.stdin.take().expect("child stdin should be opened");
        cmdstdin.write_all(&peek_buffer)?;
        io::copy(&mut stdin, &mut cmdstdin)?;
    }

    Ok(())
}
