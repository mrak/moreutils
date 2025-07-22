use std::{
    ffi::OsString,
    fmt::Display,
    fs::File,
    io::{self, BufReader, Read},
    process,
};

use crate::common::{OsLinesExt, RingBuffer};

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

#[derive(Debug)]
enum Utf8ParseError {
    Utf8(
        usize,   // line number
        usize,   // chars into line
        usize,   // total bytes so far
        Vec<u8>, // trailing context
        Vec<u8>, // unfinished codepoint bytes
        Vec<u8>, // forward context
        String,  // error message
    ),
    Io(io::Error),
}

impl From<io::Error> for Utf8ParseError {
    fn from(value: io::Error) -> Self {
        Utf8ParseError::Io(value)
    }
}

impl Display for Utf8ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
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

fn parse_args() -> Result<Options, lexopt::Error> {
    use lexopt::prelude::*;
    let mut options = Options::default();
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('h') | Long("help") => {
                usage();
                process::exit(0);
            }
            Short('i') | Long("invert") => options.display_mode = DisplayMode::Invert,
            Short('l') | Long("list") => options.display_mode = DisplayMode::List,
            Short('q') | Long("quiet") => options.display_mode = DisplayMode::Quiet,
            Short('v') | Long("verbose") => options.display_mode = DisplayMode::Verbose,
            Value(val) => options.files.push(val),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(options)
}

pub fn isutf8() -> io::Result<()> {
    let mut options: Options = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        process::exit(1);
    });

    if options.files.is_empty() {
        let stdin = io::stdin();
        let stdin = stdin.lock();

        options.files = stdin
            .os_lines()
            .collect::<Result<Vec<OsString>, io::Error>>()?;
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
                        let mut trailing_hex = byte_vec_hex(&trailing);
                        let mut context_hex = byte_vec_hex(&context);
                        let mut forward_hex = byte_vec_hex(&forward);
                        let mut trailing_ascii = byte_vec_ascii(&trailing);
                        let mut context_ascii = byte_vec_ascii(&context);
                        let mut forward_ascii = byte_vec_ascii(&forward);
                        println!(
                            "{trailing_hex} {context_hex} {forward_hex}  | {trailing_ascii}{context_ascii}{forward_ascii}"
                        );
                        trailing_hex = String::from(" ").repeat(trailing_hex.len());
                        context_hex = String::from("^").repeat(context_hex.len());
                        forward_hex = String::from(" ").repeat(forward_hex.len());
                        trailing_ascii = String::from(" ").repeat(trailing_ascii.len());
                        context_ascii = String::from("^").repeat(context_ascii.len());
                        forward_ascii = String::from(" ").repeat(forward_ascii.len());
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
    // https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-3/#G27506
    // Unicode 16.0.0 Core Spec, Chapter 3,
    // § 3.9.3, Table 3-7. Well-Formed UTF-8 Byte Sequences
    // +---------------------+--------+--------+--------+--------+
    // | Code Points / Bytes | First  | Second | Third  | Fourth |
    // +---------------------+--------+--------+--------+--------+
    // | U+0000..U+007F      | 00..7F |        |        |        |
    // | U+0080..U+07FF      | C2..DF | 80..BF |        |        |
    // | U+0800..U+0FFF      | E0     | A0..BF | 80..BF |        |
    // | U+1000..U+CFFF      | E1..EC | 80..BF | 80..BF |        |
    // | U+D000..U+D7FF      | ED     | 80..9F | 80..BF |        |
    // | U+E000..U+FFFF      | EE..EF | 80..BF | 80..BF |        |
    // | U+10000..U+3FFFF    | F0     | 90..BF | 80..BF | 80..BF |
    // | U+40000..U+FFFFF    | F1..F3 | 80..BF | 80..BF | 80..BF |
    // | U+100000..U+10FFFF  | F4     | 80..8F | 80..BF | 80..BF |
    // +---------------------+--------+--------+--------+--------+
    enum Utf8 {
        Base,
        Seq2Seen1(u8),
        Seq3Seen1(u8),
        Seq3Seen2(u8, u8),
        Seq4Seen1(u8),
        Seq4Seen2(u8, u8),
        Seq4Seen3(u8, u8, u8),
    }

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
        let mode = match (&mode, byte) {
            (Utf8::Base, b'\x0a') => {
                lines += 1;
                chars = 0;
                Ok(Utf8::Base)
            }
            (Utf8::Base, b'\x00'..=b'\x7F') => Ok(Utf8::Base),
            (Utf8::Base, b'\xC2'..=b'\xDF') => {
                bytes = count;
                Ok(Utf8::Seq2Seen1(byte))
            }
            (Utf8::Base, b'\xE0'..=b'\xEF') => {
                bytes = count;
                Ok(Utf8::Seq3Seen1(byte))
            }
            (Utf8::Base, b'\xF0'..=b'\xF4') => {
                bytes = count;
                Ok(Utf8::Seq4Seen1(byte))
            }
            (Utf8::Base, _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![byte],
                vec![],
                String::from("Expecting bytes in the following ranges: 00..7F C2..F4."),
            )),

            (Utf8::Seq2Seen1(b), b'\x80'..=b'\xBF') => {
                trailing_context.insert(*b);
                Ok(Utf8::Base)
            }
            (Utf8::Seq2Seen1(b), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between C2 and DF, expecting a 2nd byte between 80 and BF",
                ),
            )),

            (Utf8::Seq3Seen1(b @ b'\xE0'), b'\xA0'..=b'\xBF') => Ok(Utf8::Seq3Seen2(*b, byte)),
            (Utf8::Seq3Seen1(b @ b'\xE0'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from("After a first byte of E0, expecting a 2nd byte between A0 and BF."),
            )),
            (Utf8::Seq3Seen1(b @ b'\xE1'..=b'\xEC'), b'\x80'..b'\xBF') => {
                Ok(Utf8::Seq3Seen2(*b, byte))
            }
            (Utf8::Seq3Seen1(b @ b'\xE1'..=b'\xEC'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between E1 and EC, expecting a 2nd byte between 80 and BF.",
                ),
            )),
            (Utf8::Seq3Seen1(b @ b'\xED'), b'\x80'..=b'\x9F') => Ok(Utf8::Seq3Seen2(*b, byte)),
            (Utf8::Seq3Seen1(b @ b'\xED'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from("After a first byte of ED, expecting a 2nd byte between 80 and 9F."),
            )),
            (Utf8::Seq3Seen1(b @ b'\xEE'..=b'\xEF'), b'\x80'..=b'\x9F') => {
                Ok(Utf8::Seq3Seen2(*b, byte))
            }
            (Utf8::Seq3Seen1(b @ b'\xEE'..=b'\xEF'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between EE and EF, expecting a 2nd byte between 80 and BF.",
                ),
            )),
            (Utf8::Seq3Seen2(a, b), b'\x80'..=b'\xBF') => {
                trailing_context.insert(*a);
                trailing_context.insert(*b);
                Ok(Utf8::Base)
            }
            (Utf8::Seq3Seen2(a, b), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*a, *b, byte],
                vec![],
                String::from(
                    "After a first byte between E0 and EF, expecting a 3nd byte between 80 and BF.",
                ),
            )),

            (Utf8::Seq4Seen1(b @ b'\xF0'), b'\x90'..b'\xBF') => Ok(Utf8::Seq4Seen2(*b, byte)),
            (Utf8::Seq4Seen1(b @ b'\xF0'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from("After a first byte of F0, expecting a 2nd byte between 90 and BF."),
            )),
            (Utf8::Seq4Seen1(b @ b'\xF1'..=b'\xF3'), b'\x80'..b'\xBF') => {
                Ok(Utf8::Seq4Seen2(*b, byte))
            }
            (Utf8::Seq4Seen1(b @ b'\xF1'..=b'\xF3'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from(
                    "After a first byte between F1 and F3, expecting a 2nd byte between 80 and BF.",
                ),
            )),
            (Utf8::Seq4Seen1(b @ b'\xF4'), b'\x80'..b'\x8F') => Ok(Utf8::Seq4Seen2(*b, byte)),
            (Utf8::Seq4Seen1(b @ b'\xF4'), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*b, byte],
                vec![],
                String::from("After a first byte of F4, expecting a 2nd byte between 80 and BF."),
            )),
            (Utf8::Seq4Seen2(a, b), b'\x80'..=b'\xBF') => Ok(Utf8::Seq4Seen3(*a, *b, byte)),
            (Utf8::Seq4Seen2(a, b), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
                vec![*a, *b, byte],
                vec![],
                String::from(
                    "After a first byte between F0 and F4, expecting a 3nd byte between 80 and BF.",
                ),
            )),
            (Utf8::Seq4Seen3(a, b, c), b'\x80'..=b'\xBF') => {
                trailing_context.insert(*a);
                trailing_context.insert(*b);
                trailing_context.insert(*c);
                Ok(Utf8::Base)
            }
            (Utf8::Seq4Seen3(a, b, c), _) => Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                vec![],
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
        Ok(Utf8::Seq2Seen1(b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.into_vec(),
            vec![b],
            vec![],
            String::from("After a first byte between C2 and DF, expecting a 2nd byte."),
        )),
        Ok(Utf8::Seq3Seen1(b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.into_vec(),
            vec![b],
            vec![],
            String::from("After a first byte between E0 and EF, two following bytes."),
        )),
        Ok(Utf8::Seq3Seen2(a, b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.into_vec(),
            vec![a, b],
            vec![],
            String::from("After a first byte between E0 and EF, two following bytes."),
        )),
        Ok(Utf8::Seq4Seen1(b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.into_vec(),
            vec![b],
            vec![],
            String::from("After a first byte between F0 and F4, three following bytes."),
        )),
        Ok(Utf8::Seq4Seen2(a, b)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.into_vec(),
            vec![a, b],
            vec![],
            String::from("After a first byte between F0 and F4, three following bytes."),
        )),
        Ok(Utf8::Seq4Seen3(a, b, c)) => Err(Utf8ParseError::Utf8(
            lines,
            chars,
            bytes,
            trailing_context.into_vec(),
            vec![a, b, c],
            vec![],
            String::from("After a first byte between F0 and F4, three following bytes."),
        )),
        Err(Utf8ParseError::Io(e)) => Err(Utf8ParseError::Io(e)),
        Err(Utf8ParseError::Utf8(lines, chars, bytes, _, context, _, message)) => {
            let forward_context = iterator.take(6).map_while(|(_, r)| r.ok()).collect();
            Err(Utf8ParseError::Utf8(
                lines,
                chars,
                bytes,
                trailing_context.into_vec(),
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
