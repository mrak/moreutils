use std::{
    env,
    ffi::OsString,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    os::unix::ffi::OsStringExt,
    process,
};
use thiserror::Error;

use crate::common::RingBuffer;

#[derive(Default, Debug)]
enum DisplayMode {
    #[default]
    Normal,
    Quiet,
    List,
    Invert,
    Verbose,
}

#[derive(Default, Debug)]
struct Options {
    display_mode: DisplayMode,
    files: Vec<OsString>,
}

#[derive(Error, Debug)]
enum Utf8ParseError {
    #[error("{6}")]
    Utf8(usize, usize, usize, Vec<u8>, Vec<u8>, Vec<u8>, String),
    #[error("{0}")]
    Io(#[from] io::Error),
}

enum Utf8 {
    Base,
    Two(u8),
    Three(u8),
    ThreeFinal(u8, u8),
    Four(u8),
    FourThird(u8, u8),
    FourFinal(u8, u8, u8),
}

fn usage() {
    println!("Usage: isutf8 [OPTION]... [FILE]...");
    println!("Check whether input files are valid UTF-8.");
    println!();
    println!("  -h, --help       display this help text and exit");
    println!("  -q, --quiet      suppress all normal output");
    println!("  -l, --list       print only names of FILEs containing invalid UTF-8");
    println!("  -i, --invert     list valid UTF-8 files instead of invalid ones");
    println!("  -v, --verbose    print detailed error (multiple lines)");
}

pub fn isutf8() -> io::Result<()> {
    let mut options = Options::default();
    let mut double_dash = false;

    for arg in env::args_os().skip(1) {
        if double_dash {
            options.files.push(arg);
            continue;
        }
        match arg.to_str() {
            Some("--") => double_dash = true,
            Some("--help") => {
                usage();
                process::exit(0);
            }
            Some("--invert") => options.display_mode = DisplayMode::Invert,
            Some("--list") => options.display_mode = DisplayMode::List,
            Some("--quiet") => options.display_mode = DisplayMode::Quiet,
            Some("--verbose") => options.display_mode = DisplayMode::Verbose,
            Some("-") => options.files.push(arg),
            Some(x) if x.starts_with("-") => {
                for flag in x.chars().skip(1) {
                    match flag {
                        'h' => {
                            usage();
                            process::exit(0);
                        }
                        'i' => options.display_mode = DisplayMode::Invert,
                        'l' => options.display_mode = DisplayMode::List,
                        'q' => options.display_mode = DisplayMode::Quiet,
                        'v' => options.display_mode = DisplayMode::Verbose,
                        _ => {
                            eprintln!("isutf8: invalid option -- {flag}");
                            usage();
                            process::exit(1);
                        }
                    }
                }
            }
            _ => options.files.push(arg),
        }
    }

    if options.files.is_empty() {
        let mut buffer: Vec<u8> = Vec::new();
        let stdin = io::stdin();
        let mut stdin = stdin.lock();

        while let Ok(c) = stdin.read_until(b'\n', &mut buffer) {
            if c == 0 {
                break;
            }
            options.files.push(OsString::from_vec(buffer.clone()));
        }
    }

    let mut exit_code = 0;

    for file in options.files {
        match validate_file(&file) {
            Ok(_) => {
                if let DisplayMode::Invert = options.display_mode {
                    println!("{}", file.display())
                }
            }
            Err(Utf8ParseError::Io(e)) => {
                return Err(e);
            }
            Err(Utf8ParseError::Utf8(lines, chars, bytes, trailing, context, forward, message)) => {
                exit_code = 1;
                match options.display_mode {
                    DisplayMode::Normal => println!(
                        "{}: line {lines}, char {chars}, byte {bytes}: {message}",
                        file.display()
                    ),
                    DisplayMode::List => println!("{}", file.display()),
                    DisplayMode::Verbose => {
                        println!(
                            "{}: line {lines}, char {chars}, byte {bytes}: {message}",
                            file.display()
                        );
                        let trailing_hex = byte_vec_hex(&trailing);
                        let context_hex = byte_vec_hex(&context);
                        let forward_hex = byte_vec_hex(&forward);
                        let trailing_ascii = byte_vec_ascii(&trailing);
                        let context_ascii = byte_vec_ascii(&context);
                        let forward_ascii = byte_vec_ascii(&forward);
                        println!(
                            "{trailing_hex} {context_hex} {forward_hex}  | {trailing_ascii}{context_ascii}{forward_ascii}"
                        );
                    }
                    _ => {}
                }
            }
        };
    }

    process::exit(exit_code);
}

fn validate_file(file: &OsString) -> Result<(), Utf8ParseError> {
    let fd = match File::open(file) {
        Ok(f) => f,
        Err(e) => {
            return Err(Utf8ParseError::Io(e));
        }
    };
    let mut trailing_context = RingBuffer::new(8);
    let mut lines = 1;
    let mut chars = 1;
    let mut bytes = 0;
    let mut iterator = BufReader::new(fd).bytes().enumerate();
    let result = iterator.try_fold(Utf8::Base, |mode, (count, byte)| {
        let byte = byte?;
        // https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-3/#G27506
        // Unicode 16.0.0 Core Spec, Chapter 3,
        // § 3.9.3, Table 3-7. Well-Formed UTF-8 Byte Sequences
        let mode = match (&mode, byte) {
            (Utf8::Base, b'\x0a') => {
                lines += 1;
                chars = 0;
                Ok(Utf8::Base)
            }
            (Utf8::Base, b'\x00'..=b'\x7F') => Ok(Utf8::Base),
            (Utf8::Base, b'\xC2'..=b'\xDF') => {
                bytes = count;
                Ok(Utf8::Two(byte))
            }
            (Utf8::Base, b'\xE0'..=b'\xEF') => {
                bytes = count;
                Ok(Utf8::Three(byte))
            }
            (Utf8::Base, b'\xF0'..=b'\xF4') => {
                bytes = count;
                Ok(Utf8::Four(byte))
            }
            (Utf8::Base, _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![byte],
                vec![],
                String::from("Expecting bytes in the following ranges: 00..7F C2..F4."),
            )),

            (Utf8::Two(b), b'\x80'..=b'\xBF') => {
                trailing_context.insert(*b);
                Ok(Utf8::Base)
            }
            (Utf8::Two(b), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between C2 and DF, expecting a 2nd byte between 80 and BF",
                ),
            )),

            (Utf8::Three(b @ b'\xE0'), b'\xA0'..=b'\xBF') => Ok(Utf8::ThreeFinal(*b, byte)),
            (Utf8::Three(b @ b'\xE0'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from("After a first byte of E0, expecting a 2nd byte between A0 and BF."),
            )),
            (Utf8::Three(b @ b'\xE1'..=b'\xEC'), b'\x80'..b'\xBF') => {
                Ok(Utf8::ThreeFinal(*b, byte))
            }
            (Utf8::Three(b @ b'\xE1'..=b'\xEC'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between E1 and EC, expecting a 2nd byte between 80 and BF.",
                ),
            )),
            (Utf8::Three(b @ b'\xED'), b'\x80'..=b'\x9F') => Ok(Utf8::ThreeFinal(*b, byte)),
            (Utf8::Three(b @ b'\xED'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from("After a first byte of ED, expecting a 2nd byte between 80 and 9F."),
            )),
            (Utf8::Three(b @ b'\xEE'..=b'\xEF'), b'\x80'..=b'\x9F') => {
                Ok(Utf8::ThreeFinal(*b, byte))
            }
            (Utf8::Three(b @ b'\xEE'..=b'\xEF'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between EE and EF, expecting a 2nd byte between 80 and BF.",
                ),
            )),
            (Utf8::ThreeFinal(a, b), b'\x80'..=b'\xBF') => {
                trailing_context.insert(*a);
                trailing_context.insert(*b);
                Ok(Utf8::Base)
            }
            (Utf8::ThreeFinal(a, b), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*a, *b, byte],
                vec![],
                String::from(
                    "After a first byte between E0 and EF, expecting a 3nd byte between 80 and BF.",
                ),
            )),

            (Utf8::Four(b @ b'\xF0'), b'\x90'..b'\xBF') => Ok(Utf8::FourThird(*b, byte)),
            (Utf8::Four(b @ b'\xF0'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from("After a first byte of F0, expecting a 2nd byte between 90 and BF."),
            )),
            (Utf8::Four(b @ b'\xF1'..=b'\xF3'), b'\x80'..b'\xBF') => Ok(Utf8::FourThird(*b, byte)),
            (Utf8::Four(b @ b'\xF1'..=b'\xF3'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between F1 and F3, expecting a 2nd byte between 80 and BF.",
                ),
            )),
            (Utf8::Four(b @ b'\xF4'), b'\x80'..b'\x8F') => Ok(Utf8::FourThird(*b, byte)),
            (Utf8::Four(b @ b'\xF4'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*b, byte],
                vec![],
                String::from("After a first byte of F4, expecting a 2nd byte between 80 and BF."),
            )),
            (Utf8::FourThird(a, b), b'\x80'..=b'\xBF') => Ok(Utf8::FourFinal(*a, *b, byte)),
            (Utf8::FourThird(a, b), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*a, *b, byte],
                vec![],
                String::from(
                    "After a first byte between F0 and F4, expecting a 3nd byte between 80 and BF.",
                ),
            )),
            (Utf8::FourFinal(a, b, c), b'\x80'..=b'\xBF') => {
                trailing_context.insert(*a);
                trailing_context.insert(*b);
                trailing_context.insert(*c);
                Ok(Utf8::Base)
            }
            (Utf8::FourFinal(a, b, c), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.as_vec(),
                vec![*a, *b, *c, byte],
                vec![],
                String::from(
                    "After a first byte between F0 and F4, expecting a 4th byte between 80 and BF.",
                ),
            )),
            _ => unreachable!(),
        };

        if let Ok(Utf8::Base) = mode {
            trailing_context.insert(byte);
            chars += 1;
            bytes = count;
        }

        mode
    });

    match result {
        Ok(Utf8::Base) => Ok(()),
        Ok(Utf8::Two(b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.as_vec(),
            vec![b],
            vec![],
            String::from("After a first byte between C2 and DF, expecting a 2nd byte."),
        )),
        Ok(Utf8::Three(b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.as_vec(),
            vec![b],
            vec![],
            String::from("After a first byte between E0 and EF, two following bytes."),
        )),
        Ok(Utf8::ThreeFinal(a, b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.as_vec(),
            vec![a, b],
            vec![],
            String::from("After a first byte between E0 and EF, two following bytes."),
        )),
        Ok(Utf8::Four(b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.as_vec(),
            vec![b],
            vec![],
            String::from("After a first byte between F0 and F4, three following bytes."),
        )),
        Ok(Utf8::FourThird(a, b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.as_vec(),
            vec![a, b],
            vec![],
            String::from("After a first byte between F0 and F4, three following bytes."),
        )),
        Ok(Utf8::FourFinal(a, b, c)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.as_vec(),
            vec![a, b, c],
            vec![],
            String::from("After a first byte between F0 and F4, three following bytes."),
        )),
        Err(Utf8ParseError::Io(e)) => Err(Utf8ParseError::Io(e)),
        Err(Utf8ParseError::Utf8(lines, chars, bytes, trailing_context, context, _, message)) => {
            let forward_context = iterator.take(6).map_while(|(_, r)| r.ok()).collect();
            Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context,
                context,
                forward_context,
                message,
            ))
        }
    }
}

fn byte_vec_ascii(u8s: &[u8]) -> String {
    u8s.iter()
        .map(|byte| match byte {
            b'\x21'..=b'\x7e' => *byte as char,
            _ => '.',
        })
        .collect::<String>()
}

fn byte_vec_hex(u8s: &[u8]) -> String {
    u8s.iter()
        .map(|byte| format!("{byte:02X?}"))
        .collect::<Vec<String>>()
        .join(" ")
}
