use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::TimeDelta;
use core::convert::From;
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
    let mut pattern = String::from(r"\b");
    pattern.push_str(r"(?<rfc3164>\w{3}(\s\d|\s\s)\d\s\d\d:\d\d:\d\d)");
    pattern.push('|');
    pattern.push_str(r"(?<rfc3339>\d\d\d\d-\d\d-\d\d[tT ]\d\d:\d\d:\d\d(Z|[+-]\d\d:?\d\d)?)");
    pattern.push('|');
    pattern.push_str(
        r"(?<rfc2822>(\w{3},?\s+)?\d{1,2}\s+\w{3}\s+\d{4}\s+\d\d:\d\d(:\d\d)?(\s+[+-]\d{4}|\s+\w{3}))",
    );
    pattern.push_str(r"\b");
    let re = Regex::new(&pattern).unwrap();

    for line in stdin.lines().map_while(|l| l.ok()) {
        let modified = re.replace(&line, |caps: &Captures| {
            let dt = if let Some(s) = caps.name("rfc3164") {
                let now = Local::now();
                let hydrated = format!("{} {}", s.as_str(), now.format("%z %Y"));
                let parsed = DateTime::parse_from_str(&hydrated, "%b %e %H:%M:%S %z %Y")
                    .expect("syslog rfc3164 format matched");
                if parsed > now {
                    parsed.with_year(now.year() - 1).unwrap()
                } else {
                    parsed
                }
            } else if let Some(s) = caps.name("rfc3339") {
                DateTime::parse_from_rfc3339(s.as_str()).expect("rfc3339 format matched")
            } else if let Some(s) = caps.name("rfc2822") {
                DateTime::parse_from_rfc2822(s.as_str()).expect("rfc2282 format matched")
            } else {
                unreachable!();
            };
            if let Some(f) = &format {
                dt.format(f).to_string()
            } else {
                time_ago(dt)
            }
        });
        println!("{}", modified);
    }
}

fn time_ago(dt: DateTime<chrono::FixedOffset>) -> String {
    let mut delta = Local::now() - DateTime::<Local>::from(dt);
    let mut result = String::from("");
    if delta.num_days() > 0 {
        result.push_str(&format!("{}d", delta.num_days()));
        delta = delta - TimeDelta::days(delta.num_days());
    }
    if delta.num_hours() > 0 {
        result.push_str(&format!("{}h", delta.num_hours()));
        delta = delta - TimeDelta::hours(delta.num_hours());
    }
    if delta.num_minutes() > 0 {
        result.push_str(&format!("{}m", delta.num_minutes()));
        delta = delta - TimeDelta::minutes(delta.num_minutes());
    }
    if delta.num_seconds() > 0 {
        result.push_str(&format!("{}s", delta.num_seconds()));
    }
    result.push_str(" ago");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn rfc2822() {
        let year = 2025;
        let month = 4;
        let day = 14;
        let hour = 21;
        let minute = 2;
        let second = 0;
        let month_name = "Apr";
        let day_name = "Mon";
        let formats = vec![
            format!(
                "{day_name}, {day} {month_name} {year} {hour:02}:{minute:02}:{second:02} +0000"
            ),
            format!("{day_name}, {day} {month_name} {year} {hour:02}:{minute:02}:{second:02} GMT"),
            format!("{day} {month_name} {year} {hour:02}:{minute:02}:{second:02} GMT"),
        ];
        let expected = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2025, 4, 14, 21, 2, 0)
            .unwrap();
        for f in formats {
            assert_eq!(expected, DateTime::parse_from_rfc2822(&f).unwrap())
        }
    }

    #[test]
    fn rfc3339() {
        let year = 2025;
        let month = 4;
        let day = 14;
        let hour = 21;
        let minute = 2;
        let second = 0;
        let formats = vec![
            format!("{year}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"),
            format!("{year}-{month:02}-{day:02}t{hour:02}:{minute:02}:{second:02}Z"),
            format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}Z"),
            format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}z"),
            format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}z"),
            format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}-00:00"),
            format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}+00:00"),
        ];
        let expected = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2025, 4, 14, 21, 2, 0)
            .unwrap();
        for f in formats {
            assert_eq!(expected, DateTime::parse_from_rfc3339(&f).unwrap())
        }
    }
}
