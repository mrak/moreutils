use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::exit;

struct Options<'a> {
    append: bool,
    file: &'a Path,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = match args.len() {
        2 => Options {
            append: args[0].eq("-a"),
            file: Path::new(&args[1]),
        },
        1 => Options {
            append: false,
            file: Path::new(&args[0]),
        },
        _ => {
            println!("sponge [-a] FILE");
            exit(1)
        }
    };

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer)?;

    OpenOptions::new()
        .create(true)
        .append(options.append)
        .write(true)
        .open(options.file)?
        .write_all(&buffer)?;

    Ok(())
}
