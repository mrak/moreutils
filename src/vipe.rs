use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::process::exit;

use crate::common;

fn usage() {
    eprintln!("Usage: vipe [--suffix=EXTENSION]");
}

pub fn vipe() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let suffix = match args.len() {
        2 if args[0].eq("--suffix") => Some(args[1].clone()),
        1 if args[0].starts_with("--suffix=") => {
            Some(args[0].strip_prefix("--suffix=").map(String::from).unwrap())
        }
        0 => None,
        _ => {
            usage();
            exit(1)
        }
    };

    let mut tmpfilename = format!("vipe_{}", std::process::id());
    if let Some(s) = suffix {
        tmpfilename.push_str(format!(".{}", s).as_ref());
    }
    let tmpfile = env::temp_dir().join(tmpfilename);

    let result = stdin_to_tmpfile(&tmpfile)
        .and_then(|_| edit_tmpfile(&tmpfile))
        .and_then(|_| tmpfile_to_stdout(&tmpfile));

    let _ = std::fs::remove_file(tmpfile);

    match result {
        Err(e) if e.kind() == io::ErrorKind::Other => {
            eprintln!("{}", e);
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

fn edit_tmpfile(tmpfile: &Path) -> io::Result<()> {
    let editor = common::get_editor();

    let tty_in = OpenOptions::new().read(true).open("/dev/tty")?;
    let tty_out = OpenOptions::new().write(true).open("/dev/tty")?;

    let status = Command::new(&editor)
        .arg(tmpfile)
        .stdin(Stdio::from(tty_in))
        .stdout(Stdio::from(tty_out))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{} exited nonzero, aborting", editor),
        ))
    }
}

fn tmpfile_to_stdout(tmpfile: &Path) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = File::open(tmpfile)?;
    io::copy(&mut reader, &mut stdout)?;
    Ok(())
}
