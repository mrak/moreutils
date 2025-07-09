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

enum Utf8Mode {
    Base,
    TwoByte,
    ThreeByteSecond(u8),
    ThreeByteThird,
    FourByteSecond(u8),
    FourByteThird,
    FourByteFourth,
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
        let mut mode = Utf8Mode::Base;
        let bytes = BufReader::new(File::open(&file)?).bytes();
        for (count, byte) in bytes.enumerate() {
            let byte = byte?;
            match (&mode, byte) {
                (Utf8Mode::Base, b'\x00'..=b'\x7F') => {}
                (Utf8Mode::Base, b'\xC2'..=b'\xDF') => mode = Utf8Mode::TwoByte,
                (Utf8Mode::TwoByte, b'\x80'..=b'\xBF') => mode = Utf8Mode::Base,
                (Utf8Mode::ThreeByteSecond(b), b'\x80'..=b'\xBF') => match (b, byte) {
                    (b'\xE0', b'\xA0'..=b'\xBF') => mode = Utf8Mode::ThreeByteThird,
                    (b'\xE1'..=b'\xEC', b'\x80'..b'\xBF') => mode = Utf8Mode::ThreeByteThird,
                    (b'\xED', b'\x80'..=b'\x9F') => mode = Utf8Mode::ThreeByteThird,
                    (b'\xEE'..=b'\xEF', b'\x80'..=b'\x9F') => mode = Utf8Mode::ThreeByteThird,
                    _ => unimplemented!(),
                },
                (Utf8Mode::ThreeByteThird, b'\x80'..b'\xBF') => mode = Utf8Mode::Base,
                (Utf8Mode::FourByteSecond(b), b'\x80'..b'\xBF') => match (b, byte) {
                    (b'\xF0', b'\x90'..b'\xBF') => mode = Utf8Mode::FourByteThird,
                    _ => unimplemented!(),
                },
                (Utf8Mode::FourByteFourth, b'\x80'..b'\xBF') => mode = Utf8Mode::Base,
                (Utf8Mode::FourByteFourth, b'\x80'..b'\xBF') => mode = Utf8Mode::Base,
                // _ => unimplemented!(),
            }
        }
    }

    Ok(())
}
