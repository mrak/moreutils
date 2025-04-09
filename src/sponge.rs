use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::exit;

fn usage() {
    eprintln!("Usage: sponge [-a] FILE");
}

pub fn sponge() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let (append, file) = match args.len() {
        2 => (args[0].eq("-a"), Path::new(&args[1])),
        1 => (false, Path::new(&args[0])),
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
        .write(true)
        .open(file)?
        .write_all(&buffer)?;

    Ok(())
}
