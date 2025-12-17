use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;
use chrono::TimeDelta;
use chrono::format::Parsed;
use chrono::format::StrftimeItems;
use core::convert::From;
use regex::Captures;
use regex::Regex;
use std::fmt::Write as FmtWrite; // Avoid conflict with io::Write
use std::io;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::StdinLock;
use std::io::StdoutLock;
use std::io::Write;
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

struct Args {
    relative: bool,
    time_mode: TimeMode,
    monotonic: bool,
    format_arg: Option<String>,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;
    let mut relative = false;
    let mut time_mode = TimeMode::Absolute;
    let mut monotonic = false;
    let mut format_arg = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('r') => relative = true,
            Short('i') => time_mode = TimeMode::Incremental,
            Short('s') => time_mode = TimeMode::SinceStart,
            Short('m') => monotonic = true,
            Value(val) if format_arg.is_none() => format_arg = Some(val.parse()?),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Args {
        relative,
        time_mode,
        monotonic,
        format_arg,
    })
}

pub fn ts() -> io::Result<()> {
    let args: Args = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        process::exit(1);
    });

    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    if args.relative {
        return time_is_relative(stdin, &mut stdout, args.format_arg);
    }

    let format_default = match args.time_mode {
        TimeMode::Absolute => String::from("%b %d %H:%M:%S"),
        _ => String::from("%H:%M:%S"),
    };
    let format = args.format_arg.unwrap_or(format_default);

    if args.monotonic {
        with_monotonic_clock(stdin, &mut stdout, args.time_mode, &format)?;
    } else {
        with_system_clock(stdin, &mut stdout, args.time_mode, &format)?;
    }
    Ok(())
}

fn with_monotonic_clock(
    stdin: StdinLock,
    stdout: &mut BufWriter<StdoutLock>,
    mode: TimeMode,
    format: &str,
) -> io::Result<()> {
    match mode {
        TimeMode::Absolute => {
            let start_mono = Instant::now();
            let start = Local::now() - start_mono.elapsed();
            for line in stdin.lines().map_while(|l| l.ok()) {
                writeln!(
                    stdout,
                    "{} {}",
                    (start + start_mono.elapsed()).format(format),
                    line
                )?;
            }
        }
        TimeMode::Incremental => {
            let mut last = Instant::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let next = Instant::now();
                let delta = next - last;
                last = next;
                writeln!(
                    stdout,
                    "{} {}",
                    (DateTime::UNIX_EPOCH + delta).format(format),
                    line
                )?;
            }
        }
        TimeMode::SinceStart => {
            let start_mono = Instant::now();
            for line in stdin.lines().map_while(|l| l.ok()) {
                writeln!(
                    stdout,
                    "{} {}",
                    (DateTime::UNIX_EPOCH + start_mono.elapsed()).format(format),
                    line
                )?;
            }
        }
    }
    Ok(())
}

fn with_system_clock(
    stdin: StdinLock,
    stdout: &mut BufWriter<StdoutLock>,
    mode: TimeMode,
    format: &str,
) -> io::Result<()> {
    match mode {
        TimeMode::Absolute => {
            for line in stdin.lines().map_while(|l| l.ok()) {
                writeln!(stdout, "{} {}", chrono::Local::now().format(format), line)?;
            }
        }
        TimeMode::Incremental => {
            let mut last = Local::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let delta = Local::now() - last;
                last = Local::now();
                writeln!(
                    stdout,
                    "{} {}",
                    (DateTime::UNIX_EPOCH + delta).format(format),
                    line
                )?;
            }
        }
        TimeMode::SinceStart => {
            let start = Local::now();

            for line in stdin.lines().map_while(|l| l.ok()) {
                let delta = Local::now() - start;
                writeln!(
                    stdout,
                    "{} {}",
                    (DateTime::UNIX_EPOCH + delta).format(format),
                    line
                )?;
            }
        }
    }
    Ok(())
}

fn time_is_relative(
    stdin: StdinLock,
    stdout: &mut BufWriter<StdoutLock>,
    format: Option<String>,
) -> io::Result<()> {
    let mut pattern = String::from(r"\b");
    pattern.push_str(r"(?<rfc3164>\w{3}(\s\d|\s\s)\d\s\d\d:\d\d:\d\d)");
    pattern.push('|');
    pattern
        .push_str(r"(?<rfc3339>\d\d\d\d-\d\d-\d\d[tT ]\d\d:\d\d:\d\d(\.\d+)?(Z|[+-]\d\d:?\d\d)?)");
    pattern.push('|');
    pattern.push_str(r"(?<lastlog>\w{3}\s\w{3}\s{1,2}\d{1,2}\s\d\d:\d\d:\d\d [+-]\d{4}\s\d{4})");
    pattern.push('|');
    pattern.push_str(
        r"(?<rfc2822>(\w{3},?\s+)?\d{1,2}\s+\w{3}\s+\d{4}\s+\d\d:\d\d(:\d\d)?(\s+[+-]\d{4}|\s+\w{3}))",
    );
    pattern.push('|');
    pattern.push_str(r"(?<unixsec>[1-9]\d{9})");
    pattern.push_str(r"\b");
    let re = Regex::new(&pattern).expect("compile static regex");

    for line in stdin.lines().map_while(|l| l.ok()) {
        let modified = re.replace(&line, |caps: &Captures| {
            let dt_result = if let Some(s) = caps.name("rfc3164") {
                // RFC3164 doesn't include year or timezone, assume current year/zone
                let now = Local::now();
                let strftime_items = StrftimeItems::new("%b %e %H:%M:%S");
                let mut parsed = Parsed::new();
                parsed
                    .set_year(now.year() as i64)
                    .and_then(|_| parsed.set_offset(now.offset().local_minus_utc() as i64))
                    .expect("Cast from i32 and from existing Local datetime");
                chrono::format::parse(&mut parsed, s.as_str(), strftime_items)
                    .expect("parsing guaranteed by regex match");
                parsed.to_datetime().ok().and_then(|datetime| {
                    // If parsed date is in the future, assume it was last year
                    if datetime > now {
                        datetime.with_year(now.year() - 1)
                    } else {
                        Some(datetime)
                    }
                })
            } else if let Some(s) = caps.name("rfc3339") {
                match DateTime::parse_from_rfc3339(s.as_str()) {
                    Ok(dt) => Some(dt),
                    Err(_) => {
                        // parse_from_rfc3339 requires a colon in the offset, so this string must
                        // not have it. Reproduce the parsing but use %z which doesn't
                        // contain the colon.
                        //
                        // Steps:
                        // 1. parse date
                        // 2. consume space, t, or T
                        // 3. parse time+offset
                        //
                        // Step (2) cannot be represented by StrftimeItems, so we
                        // need to split into two parse and skip one character inbetween
                        let mut parsed = Parsed::new();
                        let date_items = StrftimeItems::new("%Y-%m-%d");
                        let mut remainder = chrono::format::parse_and_remainder(
                            &mut parsed,
                            s.as_str(),
                            date_items,
                        )
                        .expect("parsing guaranteed by regex match");
                        // consume t or T or space, guaranteed by regex match
                        remainder = &remainder[1..];
                        let time_items = StrftimeItems::new("%H:%M:%S%z");
                        chrono::format::parse_and_remainder(&mut parsed, remainder, time_items)
                            .expect("parsing guaranteed by regex match");
                        parsed.to_datetime().ok()
                    }
                }
            } else if let Some(s) = caps.name("rfc2822") {
                DateTime::parse_from_rfc2822(s.as_str()).ok()
            } else if let Some(s) = caps.name("lastlog") {
                DateTime::parse_from_str(s.as_str(), "%a %b %e %H:%M:%S %z %Y").ok()
            } else if let Some(s) = caps.name("unixsec") {
                DateTime::parse_from_str(s.as_str(), "%s").ok()
            } else {
                None // Should be unreachable due to regex structure
            };

            // If parsing succeeded, format it; otherwise, keep original string
            if let Some(dt) = dt_result {
                if let Some(f) = &format {
                    dt.format(f).to_string()
                } else {
                    time_ago(dt.into()) // Convert to DateTime<Local> for time_ago
                }
            } else {
                // Parsing failed, return the original matched text
                caps.get(0).map_or("", |m| m.as_str()).to_string()
            }
        });
        writeln!(stdout, "{modified}")?;
    }
    Ok(())
}

fn time_ago(dt: DateTime<Local>) -> String {
    let now = Local::now();
    let mut delta = now - dt;

    // Handle cases slightly in the future due to clock skew or rounding
    if delta < TimeDelta::zero() {
        return "just now".to_string();
    }

    let mut terms = 0; // Limit to two terms, i.e. "Xd Yhr ago"
    let mut result = String::with_capacity(20); // Pre-allocate roughly

    if delta.num_days() > 0 {
        terms += 1;
        write!(result, "{}d ", delta.num_days()).unwrap();
        delta = delta - TimeDelta::days(delta.num_days());
    }
    if delta.num_hours() > 0 {
        terms += 1;
        write!(result, "{}h ", delta.num_hours()).unwrap();
        delta = delta - TimeDelta::hours(delta.num_hours());
    }
    if terms < 2 && delta.num_minutes() > 0 {
        write!(result, "{}m ", delta.num_minutes()).unwrap();
        delta = delta - TimeDelta::minutes(delta.num_minutes());
    }
    if terms < 2 && delta.num_seconds() > 0 {
        write!(result, "{}s ", delta.num_seconds()).unwrap();
    }

    if result.is_empty() {
        "just now".to_string()
    } else {
        result.push_str("ago");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    #[test]
    fn lastlog() {
        let year = 2025;
        let day = 1;
        let hour = 21;
        let minute = 2;
        let second = 0;
        let month_name = "Apr";
        let day_name = "Tue";
        let formats = vec![format!(
            "{day_name} {month_name} {day} {hour:02}:{minute:02}:{second:02} +0000 {year}"
        )];
        let expected = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2025, 4, 1, 21, 2, 0)
            .unwrap();
        for f in formats {
            assert_eq!(
                expected,
                DateTime::parse_from_str(&f, "%a %b %e %H:%M:%S %z %Y").unwrap()
            )
        }
    }

    #[test]
    fn rfc2822() {
        let year = 2025;
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
