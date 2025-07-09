use std::{
    env,
    ffi::OsString,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    os::unix::ffi::OsStringExt,
    process,
};

#[derive(Default, Debug)]
struct Options {
    invert: bool,
    list: bool,
    quiet: bool,
    verbose: bool,
    files: Vec<OsString>,
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
        validate_file(&file)?;
    }

    Ok(())
}

fn validate_file(file: &OsString) -> io::Result<()> {
    let mut mode = Utf8::Base;
    let bytes = BufReader::new(File::open(file)?).bytes();
    for (count, byte) in bytes.enumerate() {
        let byte = byte?;
        mode = match (&mode, byte) {
            (Utf8::Base, b'\x00'..=b'\x7F') => Utf8::Base,
            (Utf8::Base, b'\xC2'..=b'\xDF') => Utf8::Two,
            (Utf8::Base, b'\xE0'..=b'\xEF') => Utf8::Three(byte),
            (Utf8::Base, b'\xF0'..=b'\xF4') => Utf8::Four(byte),

            (Utf8::Two, b'\x80'..=b'\xBF') => Utf8::Base,

            (Utf8::Three(b'\xE0'), b'\xA0'..=b'\xBF') => Utf8::ThreeFinal,
            (Utf8::Three(b'\xE1'..=b'\xEC'), b'\x80'..b'\xBF') => Utf8::ThreeFinal,
            (Utf8::Three(b'\xED'), b'\x80'..=b'\x9F') => Utf8::ThreeFinal,
            (Utf8::Three(b'\xEE'..=b'\xEF'), b'\x80'..=b'\x9F') => Utf8::ThreeFinal,
            (Utf8::ThreeFinal, b'\x80'..=b'\xBF') => Utf8::Base,

            (Utf8::Four(b'\xF0'), b'\x90'..b'\xBF') => Utf8::FourThird,
            (Utf8::Four(b'\xF1'..=b'\xF3'), b'\x80'..b'\xBF') => Utf8::FourThird,
            (Utf8::Four(b'\xF4'), b'\x80'..b'\x8F') => Utf8::FourThird,
            (Utf8::FourThird, b'\x80'..=b'\xBF') => Utf8::FourFinal,
            (Utf8::FourFinal, b'\x80'..=b'\xBF') => Utf8::Base,
            _ => {
                eprintln!("{file:?} is invalid UTF-8 at byte {count}");
                break;
            }
        }
    }
    Ok(())
}
