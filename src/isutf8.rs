use anyhow::Result;
use std::{
    env,
    ffi::OsString,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    os::unix::ffi::OsStringExt,
    process,
};
use thiserror::Error;

#[derive(Default, Debug)]
struct Options {
    invert: bool,
    list: bool,
    quiet: bool,
    verbose: bool,
    files: Vec<OsString>,
}

#[derive(Error, Debug)]
enum ValidationError {
    #[error("{1}")]
    Utf8(usize, usize, usize, String),
    #[error("{0}")]
    Io(#[from] io::Error),
}

enum Utf8 {
    Base,
    Two,
    Three(u8),
    ThreeFinal,
    Four(u8),
    FourThird,
    FourFinal,
}

fn usage() {}

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
            Some("--invert") => options.invert = true,
            Some("--list") => options.list = true,
            Some("--quiet") => options.quiet = true,
            Some("--verbose") => options.verbose = true,
            Some("-") => options.files.push(arg),
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

    for file in options.files {
        match validate_file(&file) {
            Ok(_) => {}
            Err(ValidationError::Io(e)) => {
                return Err(e);
            }
            Err(ValidationError::Utf8(lines, chars, bytes, message)) => {
                println!(
                    "{}: line {lines}, char {chars}, byte {bytes}: {message}",
                    file.display()
                );
            }
        };
    }

    Ok(())
}

fn validate_file(file: &OsString) -> Result<(), ValidationError> {
    let fd = match File::open(file) {
        Ok(f) => f,
        Err(e) => {
            return Err(ValidationError::Io(e));
        }
    };
    let mut lines = 1;
    let mut chars = 1;
    let mut bytes = 0;
    let result = BufReader::new(fd)
        .bytes()
        .enumerate()
        .try_fold(Utf8::Base, |mode, (count, byte)| {
            let byte = byte?;
            // https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-3/#G27506
            // Unicode 16.0.0 Core Spec, Chapter 3,
            // § 3.9.3, Table 3-7. Well-Formed UTF-8 Byte Sequences
            Ok(match (&mode, byte) {
                (Utf8::Base, b'\x0a') => {
                    lines += 1;
                    chars = 1;
                    bytes = count;
                    Utf8::Base
                }
                (Utf8::Base, b'\x00'..=b'\x7F') => {
                    chars += 1;
                    bytes = count;
                    Utf8::Base
                }
                (Utf8::Base, b'\xC2'..=b'\xDF') => { bytes = count; Utf8::Two },
                (Utf8::Base, b'\xE0'..=b'\xEF') => { bytes = count; Utf8::Three(byte) },
                (Utf8::Base, b'\xF0'..=b'\xF4') => { bytes = count; Utf8::Four(byte) },
                (Utf8::Base, _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "Expecting bytes in the following ranges: 00..7F C2..F4."
                        ),
                    ));
                }

                (Utf8::Two, b'\x80'..=b'\xBF') => {
                    chars += 1;
                    bytes = count;
                    Utf8::Base
                }
                (Utf8::Two, _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte between C2 and DF, expecting a 2nd byte between 80 and BF"
                        ),
                    ));
                }

                (Utf8::Three(b'\xE0'), b'\xA0'..=b'\xBF') => Utf8::ThreeFinal,
                (Utf8::Three(b'\xE0'), _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte of E0, expecting a 2nd byte between A0 and BF."
                        ),
                    ));
                }
                (Utf8::Three(b'\xE1'..=b'\xEC'), b'\x80'..b'\xBF') => Utf8::ThreeFinal,
                (Utf8::Three(b'\xE1'..=b'\xEC'), _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte between E1 and EC, expecting a 2nd byte between 80 and BF."
                        ),
                    ));
                }
                (Utf8::Three(b'\xED'), b'\x80'..=b'\x9F') => Utf8::ThreeFinal,
                (Utf8::Three(b'\xED'), _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte of ED, expecting a 2nd byte between 80 and 9F."
                        ),
                    ));
                }
                (Utf8::Three(b'\xEE'..=b'\xEF'), b'\x80'..=b'\x9F') => Utf8::ThreeFinal,
                (Utf8::Three(b'\xEE'..=b'\xEF'), _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte between EE and EF, expecting a 2nd byte between 80 and BF."
                        ),
                    ));
                }
                (Utf8::ThreeFinal, b'\x80'..=b'\xBF') => {
                    chars += 1;
                    bytes = count;
                    Utf8::Base
                }
                (Utf8::ThreeFinal, _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte between E0 and EF, expecting a 3nd byte between 80 and BF."
                        ),
                    ));
                }

                (Utf8::Four(b'\xF0'), b'\x90'..b'\xBF') => Utf8::FourThird,
                (Utf8::Four(b'\xF0'), _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte of F0, expecting a 2nd byte between 90 and BF."
                        ),
                    ));
                }
                (Utf8::Four(b'\xF1'..=b'\xF3'), b'\x80'..b'\xBF') => Utf8::FourThird,
                (Utf8::Four(b'\xF1'..=b'\xF3'), _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte between F1 and F3, expecting a 2nd byte between 80 and BF."
                        ),
                    ));
                }
                (Utf8::Four(b'\xF4'), b'\x80'..b'\x8F') => Utf8::FourThird,
                (Utf8::Four(b'\xF4'), _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte of F4, expecting a 2nd byte between 80 and BF."
                        ),
                    ));
                }
                (Utf8::FourThird, b'\x80'..=b'\xBF') => Utf8::FourFinal,
                (Utf8::FourThird, _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte between F0 and F4, expecting a 3nd byte between 80 and BF."
                        ),
                    ));
                }
                (Utf8::FourFinal, b'\x80'..=b'\xBF') => {
                    chars += 1;
                    bytes = count;
                    Utf8::Base
                }
                (Utf8::FourFinal, _) => {
                    return Err(ValidationError::Utf8(
                        lines, chars, bytes,
                        String::from(
                            "After a first byte between F0 and F4, expecting a 4th byte between 80 and BF."
                        ),
                    ));
                }
                _ => unreachable!(),
            })
        });

    match result {
        Ok(Utf8::Base) => Ok(()),
        Ok(Utf8::Two) => Err(ValidationError::Utf8(
            lines,
            chars,
            bytes,
            String::from("After a first byte between C2 and DF, expecting a 2nd byte."),
        )),
        Ok(Utf8::Three(_) | Utf8::ThreeFinal) => Err(ValidationError::Utf8(
            lines,
            chars,
            bytes,
            String::from("After a first byte between E0 and EF, two following bytes."),
        )),
        Ok(Utf8::Four(_) | Utf8::FourThird | Utf8::FourFinal) => Err(ValidationError::Utf8(
            lines,
            chars,
            bytes,
            String::from("After a first byte between F0 and F4, three following bytes."),
        )),
        Err(e) => Err(e),
    }
}
