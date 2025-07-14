use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

fn usage() {
    eprintln!("Usage: sponge [-a] FILE");
}

pub fn sponge() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    let (append, file) = match (args.next(), args.next()) {
        (Some(a), Some(f)) if a == "-a" => (true, PathBuf::from(&f)),
        (Some(f), None) => (false, PathBuf::from(&f)),
        _ => {
            usage();
            exit(1)
        }
    };

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer)?;

    OpenOptions::new()
        .create(true)
        .append(append)
        .truncate(!append)
        .write(true)
        .open(file)?
        .write_all(&buffer)?;

    Ok(())
}
