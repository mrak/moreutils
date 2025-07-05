use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, process};

fn usage() {
    eprintln!("Usage: chronic [-ev] COMMAND...");
}

pub fn chronic() -> io::Result<()> {
    let mut verbose = false;
    let mut trigger_stderr = false;
    let args = env::args_os().skip(1);
    let mut cmdline: Vec<OsString> = Vec::new();
    let mut in_cmd = false;

    for arg in args {
        if in_cmd {
            cmdline.push(arg);
            continue;
        }
        if arg
            .as_bytes()
            .first()
            .map(|&b| b == b'-')
            .unwrap_or_else(|| false)
        {
            for b in arg.as_bytes().iter().skip(1) {
                match b {
                    b'e' => trigger_stderr = true,
                    b'v' => verbose = true,
                    _ => {
                        usage();
                        process::exit(255);
                    }
                }
            }
        } else {
            in_cmd = true;
            cmdline.push(arg);
        }
    }

    let cmd = match cmdline.first() {
        Some(c) => c,
        None => {
            usage();
            process::exit(255);
        }
    };
    let cmdargs = &cmdline[1..];

    let tmp_stdout_filename = env::temp_dir().join(format!("chronic_{}.out", std::process::id()));
    let tmp_stderr_filename = env::temp_dir().join(format!("chronic_{}.err", std::process::id()));

    let tmp_stdout = File::create(&tmp_stdout_filename)?;
    let tmp_stderr = File::create(&tmp_stderr_filename)?;

    let result = match Command::new(cmd)
        .args(cmdargs)
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
            if trigger_stderr && File::metadata(&File::open(&tmp_stderr_filename)?)?.size() > 0 {
                output(verbose, 0, &tmp_stdout_filename, &tmp_stderr_filename)?;
            }
            process::exit(0);
        }
        Some(code) => {
            output(verbose, code, &tmp_stdout_filename, &tmp_stderr_filename)?;
            process::exit(code);
        }
        None => {
            // Killed by signal?
            let code = result.signal().map(|s| 128 + s).unwrap_or(1);
            output(verbose, code, &tmp_stdout_filename, &tmp_stderr_filename)?;
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
