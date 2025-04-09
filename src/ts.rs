use std::env;
use std::io;
use std::io::BufRead;
use std::process;

fn usage() {
    eprintln!("Usage: ts [-r] [-i|-s] [-m] [FORMAT]");
}

enum TimeMode {
    Absolute,
    Incremental,
    SinceStart,
}

pub fn ts() -> io::Result<()> {
    let mut relative = false;
    let mut time_mode = TimeMode::Absolute;
    let mut monotonic = false;
    let mut format_arg = None;
    let mut double_dash = false;

    let args = env::args().skip(1);
    for arg in args {
        match arg.as_ref() {
            "--" => double_dash = true,
            x if x.starts_with("-") && !double_dash => {
                for flag in x.chars().skip(1) {
                    match flag {
                        'r' => relative = true,
                        'i' => time_mode = TimeMode::Incremental,
                        's' => time_mode = TimeMode::SinceStart,
                        'm' => monotonic = true,
                        c => {
                            eprintln!("Unknown option: {}", c);
                            usage();
                            process::exit(1);
                        }
                    }
                }
            }
            x => format_arg = Some(x.to_owned()),
        }
    }

    let format_default = match time_mode {
        TimeMode::Absolute => String::from("%b %d %H:%M:%S"),
        _ => String::from("%H:%M:%S"),
    };
    let format = format_arg.unwrap_or(format_default);

    let stdin = io::stdin();
    let stdin = stdin.lock();

    match time_mode {
        TimeMode::Absolute => {
            for line in stdin.lines().map_while(|l| l.ok()) {
                println!("{} {}", chrono::Local::now().format(&format), line);
            }
        }
        TimeMode::Incremental => {
            let mut last = chrono::Local::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let delta = chrono::Local::now() - last;
                last = chrono::Local::now();
                println!(
                    "{} {}",
                    (chrono::NaiveDateTime::UNIX_EPOCH + delta).format(&format),
                    line
                );
            }
        }
        TimeMode::SinceStart => {
            let last = chrono::Local::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let delta = chrono::Local::now() - last;
                println!(
                    "{} {}",
                    (chrono::NaiveDateTime::UNIX_EPOCH + delta).format(&format),
                    line
                );
            }
        }
    }
    Ok(())
}
