pub mod errno_generated;
pub use errno_generated::{ERRNOS, Errno};
use regex::Regex;
use std::{io, process};

fn usage() {
    eprintln!("Usage: errno [-lsS] [--list] [--search] [--search-all-locales] [keyword]");
}

fn parse_args() -> Result<Mode, lexopt::Error> {
    use lexopt::prelude::*;
    let mut mode: Option<Mode> = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('l') | Long("list") => mode = Some(Mode::List),
            Short('s') | Long("search") => mode = Some(Mode::Search(parser.value()?.parse()?)),
            Short('S') | Long("search-all-locales") => {
                mode = Some(Mode::SearchAllLocale(parser.value()?.parse()?))
            }
            Value(val) if mode.is_none() => {
                mode = Some(if let Ok(i) = val.parse::<i32>() {
                    Mode::LookupCode(i)
                } else {
                    Mode::LookupName(val.parse()?)
                });
            }
            _ => return Err(arg.unexpected()),
        }
    }
    mode.ok_or(lexopt::Error::from("expected argument"))
}

pub fn errno() -> io::Result<()> {
    let re = Regex::new(r" \(os error \d+\)").expect("compile static regex");
    let mode = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        process::exit(1);
    });

    match mode {
        Mode::List => {
            for errno in ERRNOS {
                let description = std::io::Error::from_raw_os_error(errno.id).to_string();
                let description = re.replace(&description, "");
                println!("{} {} {}", errno.name, errno.id, description,);
            }
        }
        Mode::LookupName(s) => match ERRNOS.iter().find(|e| e.name == s.to_uppercase()) {
            Some(errno) => {
                print_errno(errno);
            }
            None => {
                println!("Unknown errno");
            }
        },
        Mode::LookupCode(c) => ERRNOS.iter().filter(|e| e.id == c).for_each(|errno| {
            print_errno(errno);
        }),
        Mode::Search(s) => ERRNOS
            .iter()
            .filter(|&e| description(e).to_lowercase().contains(&s.to_lowercase()))
            .for_each(|errno| {
                print_errno(errno);
            }),
        Mode::SearchAllLocale(_) => unimplemented!(),
    }

    Ok(())
}

fn print_errno(errno: &Errno) {
    println!("{} {} {}", errno.name, errno.id, description(errno));
}

fn description(errno: &Errno) -> String {
    let re = Regex::new(r" \(os error \d+\)").unwrap();
    let description = std::io::Error::from_raw_os_error(errno.id).to_string();
    let description = re.replace(&description, "");
    description.to_string()
}

enum Mode {
    LookupName(String),
    LookupCode(i32),
    List,
    Search(String),
    #[allow(dead_code)] // not implemented yet
    SearchAllLocale(String),
}
