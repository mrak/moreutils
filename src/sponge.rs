use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

fn usage() {
    eprintln!("Usage: sponge [-a] FILE");
}

struct Args {
    append: bool,
    file: PathBuf,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut append = false;
    let mut file: Option<PathBuf> = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('a') => append = true,
            Value(val) if file.is_none() => file = Some(PathBuf::from(&val)),
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        append,
        file: file.ok_or("missing argument FILE")?,
    })
}

pub fn sponge() -> io::Result<()> {
    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        exit(1);
    });

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer)?;

    OpenOptions::new()
        .create(true)
        .append(args.append)
        .truncate(!args.append)
        .write(true)
        .open(args.file)?
        .write_all(&buffer)?;

    Ok(())
}
