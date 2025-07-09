use std::{env, ffi::OsString, io, process};

#[derive(Default)]
struct Options {
    invert: bool,
    list: bool,
    quiet: bool,
    verbose: bool,
}

fn usage() {}

pub fn isutf8() -> io::Result<()> {
    let mut options = Options::default();
    let mut double_dash = false;
    let mut files: Vec<OsString> = Vec::new();

    for arg in env::args_os().skip(1) {
        if double_dash {
            files.push(arg);
            continue;
        }
        match arg.to_str() {
            Some("--") => double_dash = true,
            Some("--help") => {
                usage();
                process::exit(0);
            }
            Some("--invert") => options.invert = true,
            Some("--list") => options.list = true,
            Some("--quiet") => options.quiet = true,
            Some("--verbose") => options.verbose = true,
            Some("-") => files.push(arg),
            Some(x) if x.starts_with("-") => {
                for flag in x.chars().skip(1) {
                    match flag {
                        'h' => {
                            usage();
                            process::exit(0);
                        }
                        'i' => options.invert = true,
                        'l' => options.list = true,
                        'q' => options.quiet = true,
                        'v' => options.verbose = true,
                        _ => {
                            eprintln!("isutf8: invalid option -- {flag}");
                            usage();
                            process::exit(1);
                        }
                    }
                }
            }
            _ => files.push(arg),
        }
    }

    if files.is_empty() {
        // TODO read filenames from STDIN
    }
    Ok(())
}
