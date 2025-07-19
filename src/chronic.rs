use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::exit;
use std::process::{Command, Stdio};
use std::{env, process};

fn usage() {
    eprintln!("Usage: chronic [-ev] COMMAND...");
}

struct Args {
    verbose: bool,
    trigger_stderr: bool,
    command: OsString,
    arguments: Vec<OsString>,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;
    let mut verbose = false;
    let mut trigger_stderr = false;
    let mut command: Option<OsString> = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('v') => verbose = true,
            Short('e') => trigger_stderr = true,
            Value(cmd) => {
                command = Some(cmd);
                break;
            }
            _ => return Err(arg.unexpected()),
        }
    }
    let arguments: Vec<OsString> = parser.raw_args()?.collect();

    Ok(Args {
        verbose,
        trigger_stderr,
        command: command.ok_or("missing argument COMMAND")?,
        arguments,
    })
}

pub fn chronic() -> io::Result<()> {
    let args: Args = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        exit(1);
    });

    let tmp_stdout_filename = env::temp_dir().join(format!("chronic_{}.out", std::process::id()));
    let tmp_stderr_filename = env::temp_dir().join(format!("chronic_{}.err", std::process::id()));

    let tmp_stdout = File::create(&tmp_stdout_filename)?;
    let tmp_stderr = File::create(&tmp_stderr_filename)?;

    let result = match Command::new(args.command)
        .args(args.arguments)
        .stdout(Stdio::from(tmp_stdout))
        .stderr(Stdio::from(tmp_stderr))
        .status()
    {
        Ok(ec) => ec,
        Err(e) => {
            eprintln!("Failed to execute command: {e}");
            process::exit(2);
        }
    };

    match result.code() {
        Some(0) => {
            if args.trigger_stderr && File::metadata(&File::open(&tmp_stderr_filename)?)?.size() > 0
            {
                output(args.verbose, 0, &tmp_stdout_filename, &tmp_stderr_filename)?;
            }
            process::exit(0);
        }
        Some(code) => {
            output(
                args.verbose,
                code,
                &tmp_stdout_filename,
                &tmp_stderr_filename,
            )?;
            process::exit(code);
        }
        None => {
            // Killed by signal?
            let code = result.signal().map(|s| 128 + s).unwrap_or(1);
            output(
                args.verbose,
                code,
                &tmp_stdout_filename,
                &tmp_stderr_filename,
            )?;
            process::exit(1);
        }
    }
}

fn output(
    verbose: bool,
    code: i32,
    stdout_filename: &Path,
    stderr_filename: &Path,
) -> io::Result<()> {
    let mut out = File::open(stdout_filename)?;
    let mut err = File::open(stderr_filename)?;
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    if verbose {
        println!("STDOUT:");
    }
    io::copy(&mut out, &mut stdout)?;
    if verbose {
        eprintln!();
        eprintln!("STDERR:");
    }
    io::copy(&mut err, &mut stderr)?;
    if verbose {
        println!();
        println!("RETVAL: {code}");
    }
    Ok(())
}
