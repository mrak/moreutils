pub mod errno_generated;
pub use errno_generated::{ERRNOS, Errno};
use regex::Regex;
use std::{env, io, process};

pub fn errno() -> io::Result<()> {
    let re = Regex::new(r" \(os error \d+\)").expect("compile static regex");
    let mut args = env::args().skip(1).peekable();
    let mode = match args.peek().map(|a| a.as_ref()) {
        Some("-l") => Mode::List,
        Some("-s") => Mode::Search(args.nth(1).expect("-s needs an argument")),
        Some("-S") => Mode::SearchAllLocale(args.nth(1).expect("-S needs an argument")),
        Some(a) => {
            if let Ok(i) = a.parse::<i32>() {
                Mode::LookupCode(i)
            } else {
                Mode::LookupName(String::from(a))
            }
        }
        None => {
            process::exit(0);
        }
    };

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
