use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::exit;

struct Options {
    append: bool,
    file: String,
}

fn usage() {
    println!("sponge [-a] FILE");
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = match args.len() {
        2 => Options {
            append: args[0].eq("-a"),
            file: args[1].clone(),
        },
        1 => Options {
            append: false,
            file: args[0].clone(),
        },
        _ => {
            usage();
            exit(1)
        }
    };

    let tmpfilename = soak(&options)?;

    fs::copy(&tmpfilename, options.file)?;
    fs::remove_file(&tmpfilename)?;

    Ok(())
}

fn soak(options: &Options) -> io::Result<PathBuf> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let tmpfile = env::temp_dir().join(format!("sponge.{}", std::process::id()));

    if options.append {
        fs::copy(&options.file, &tmpfile)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&tmpfile)?;
    let mut writer = BufWriter::new(file);

    loop {
        let buffer = stdin.fill_buf()?;
        let buflen = buffer.len();
        if buffer.is_empty() {
            break;
        }
        writer.write_all(buffer)?;
        stdin.consume(buflen);
    }

    writer.flush()?;
    Ok(tmpfile)
}
