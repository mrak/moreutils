use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

struct Options<'a> {
    append: bool,
    file: &'a Path,
}

fn usage() {
    println!("sponge [-a] FILE");
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
            usage();
            exit(1)
        }
    };

    let tmpfilename = soak(&options)?;
    squeeze(&tmpfilename, options.file)?;

    Ok(())
}

fn squeeze(tmpfilename: &PathBuf, target: &Path) -> io::Result<()> {
    let original_mode = match fs::metadata(target) {
        Ok(meta) => Some(meta.permissions().mode()),
        Err(_) => None,
    };

    if fs::rename(tmpfilename, target).is_err() {
        fs::copy(tmpfilename, target)?;
        fs::remove_file(tmpfilename)?;
    }

    if let Some(mode) = original_mode {
        let mut permissions = fs::metadata(target)?.permissions();
        permissions.set_mode(mode);
    }
    Ok(())
}

fn soak(options: &Options) -> io::Result<PathBuf> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let tmpfile = env::temp_dir().join(format!("sponge.{}", std::process::id()));

    if options.append {
        fs::copy(options.file, &tmpfile)?;
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
