use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::process::exit;

use crate::common;

fn usage() {
    eprintln!("Usage: vipe [--suffix=EXTENSION]");
}

type Suffix = Option<OsString>;

fn parse_args() -> Result<Suffix, lexopt::Error> {
    use lexopt::prelude::*;
    let mut suffix = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("suffix") => suffix = Some(parser.value()?),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(suffix)
}

pub fn vipe() -> io::Result<()> {
    let suffix = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        exit(1);
    });

    let mut tmpfile = env::temp_dir().join(format!("vipe_{}", std::process::id()));
    if let Some(s) = suffix {
        tmpfile.set_extension(&s);
    }

    let result = stdin_to_tmpfile(&tmpfile)
        .and_then(|_| common::edit_tmpfile(&tmpfile))
        .and_then(|_| tmpfile_to_stdout(&tmpfile));

    let _ = std::fs::remove_file(tmpfile);

    match result {
        Err(e) if e.kind() == io::ErrorKind::Other => {
            eprintln!("{e}");
            exit(1)
        }
        _ => result,
    }
}

fn stdin_to_tmpfile(tmpfile: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(tmpfile)?;
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    io::copy(&mut stdin, &mut file)?;
    Ok(())
}

fn tmpfile_to_stdout(tmpfile: &Path) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = File::open(tmpfile)?;
    io::copy(&mut reader, &mut stdout)?;
    Ok(())
}
