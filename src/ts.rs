use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::prelude::*;
use regex::Captures;
use regex::Regex;
use std::env;
use std::io;
use std::io::BufRead;
use std::io::StdinLock;
use std::process;
use std::time::Instant;

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

    let stdin = io::stdin();
    let stdin = stdin.lock();

    if relative {
        time_is_relative(stdin, format_arg);
        return Ok(());
    }

    let format_default = match time_mode {
        TimeMode::Absolute => String::from("%b %d %H:%M:%S"),
        _ => String::from("%H:%M:%S"),
    };
    let format = format_arg.unwrap_or(format_default);

    if monotonic {
        with_monotonic_clock(stdin, time_mode, &format);
    } else {
        with_system_clock(stdin, time_mode, &format);
    }
    Ok(())
}

fn with_monotonic_clock(stdin: StdinLock, mode: TimeMode, format: &str) {
    match mode {
        TimeMode::Absolute => {
            let start_mono = Instant::now();
            let start = Local::now() - start_mono.elapsed();
            for line in stdin.lines().map_while(|l| l.ok()) {
                println!("{} {}", (start + start_mono.elapsed()).format(format), line);
            }
        }
        TimeMode::Incremental => {
            let mut last = Instant::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let next = Instant::now();
                let delta = next - last;
                last = next;
                println!(
                    "{} {}",
                    (NaiveDateTime::UNIX_EPOCH + delta).format(format),
                    line
                );
            }
        }
        TimeMode::SinceStart => {
            let start_mono = Instant::now();
            for line in stdin.lines().map_while(|l| l.ok()) {
                println!(
                    "{} {}",
                    (NaiveDateTime::UNIX_EPOCH + start_mono.elapsed()).format(format),
                    line
                );
            }
        }
    }
}

fn with_system_clock(stdin: StdinLock, mode: TimeMode, format: &str) {
    match mode {
        TimeMode::Absolute => {
            for line in stdin.lines().map_while(|l| l.ok()) {
                println!("{} {}", chrono::Local::now().format(format), line);
            }
        }
        TimeMode::Incremental => {
            let mut last = Local::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let delta = Local::now() - last;
                last = Local::now();
                println!(
                    "{} {}",
                    (chrono::NaiveDateTime::UNIX_EPOCH + delta).format(format),
                    line
                );
            }
        }
        TimeMode::SinceStart => {
            let last = Local::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let delta = Local::now() - last;
                println!(
                    "{} {}",
                    (chrono::NaiveDateTime::UNIX_EPOCH + delta).format(format),
                    line
                );
            }
        }
    }
}

fn time_is_relative(stdin: StdinLock, format: Option<String>) {
    let syslog = Regex::new(r"\b(?<syslog>\w{3}\s{1,2}\d{1,2}\s{1,2}\d\d:\d\d:\d\d)\b").unwrap();
    for line in stdin.lines().map_while(|l| l.ok()) {
        let mut changed = false;
        syslog.replace(&line, |caps: &Captures| {
            changed = true;
            let dt = DateTime::parse_from_str(&caps["syslog"], "").expect("syslog format matched");
            if let Some(f) = &format {
                dt.format(f).to_string()
            } else {
                caps["syslog"].to_string()
            }
        });
        if changed {
            continue;
        }
    }
}
