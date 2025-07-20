use std::env;
use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::process::exit;

fn usage() {
    eprintln!("parallel [OPTIONS] command -- arguments");
    eprintln!("        for each argument, run command iwth argument, in parallel");
    eprintln!("parallel [OPTIONS] -- commands");
    eprintln!("        run specified commands in parallel");
}

#[derive(Debug)]
enum CommandMode {
    Single(OsString, Vec<OsString>, Vec<OsString>),
    Multi(Vec<OsString>),
}

pub fn parallel() -> io::Result<()> {
    let mut interpolate = false;
    let mut n_args: usize = 1;
    let mut maxload: Option<f64> = None;
    let mut maxjobs: Option<usize> = None;

    let mut args = env::args_os().skip(1).peekable();

    while let Some(arg) = args.peek() {
        if arg == "--" {
            break;
        }

        if arg.as_bytes().first().is_none_or(|b| *b != b'-') {
            break;
        }

        let arg = args.next().unwrap(); // consume peeked arg
        match arg.to_str() {
            Some("-h") => {
                usage();
                exit(0);
            }
            Some("-i") => interpolate = true,
            Some("-n") => {
                n_args = args
                    .next()
                    .and_then(|os_str| os_str.to_str().and_then(|s| s.parse::<usize>().ok()))
                    .unwrap_or_else(|| {
                        eprintln!("parallel: -n requires a positive integer argument");
                        exit(1);
                    })
            }
            Some("-l") => {
                maxload = Some(
                    args.next()
                        .and_then(|os_str| os_str.to_str().and_then(|s| s.parse::<f64>().ok()))
                        .unwrap_or_else(|| {
                            eprintln!("parallel: -l requires a number argument");
                            exit(1);
                        }),
                )
            }
            Some("-j") => {
                maxjobs = Some(
                    args.next()
                        .and_then(|os_str| os_str.to_str().and_then(|s| s.parse::<usize>().ok()))
                        .unwrap_or_else(|| {
                            eprintln!("parallel: -j requires a positive integer argument");
                            exit(1);
                        }),
                )
            }
            _ => {
                eprintln!("parallel: invalid option -- '{}'", arg.display());
                usage();
                exit(1);
            }
        }
    }

    let mode = if let Some("--") = args.peek().and_then(|a| a.to_str()) {
        CommandMode::Multi(args.skip(1).collect())
    } else {
        let command = args.next().unwrap();
        let (fixed_args, parallel_args) = split_args(args);
        CommandMode::Single(command, fixed_args, parallel_args)
    };

    println!("-i {interpolate} -l {maxload:?} -j {maxjobs:?} -n {n_args} {mode:?}");
    Ok(())
}

fn split_args<I>(iter: I) -> (Vec<OsString>, Vec<OsString>)
where
    I: Iterator<Item = OsString>,
{
    let mut before: Vec<OsString> = Vec::new();
    let mut after: Vec<OsString> = Vec::new();
    let mut double_dash = false;

    for item in iter {
        if double_dash {
            after.push(item);
        } else if item == "--" {
            double_dash = true;
        } else {
            before.push(item);
        }
    }

    (before, after)
}
