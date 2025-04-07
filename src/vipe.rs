use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;

use crate::common;

fn usage() {
    println!("Usage: vipe [--suffix=extension]");
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

    stdin_to_tmpfile(&tmpfile)?;
    edit_tmpfile(&tmpfile)?;
    tmpfile_to_stdout(&tmpfile)?;

    std::fs::remove_file(tmpfile)?;

    Ok(())
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
        eprintln!("{} exited nonzero, aborting", editor);
        exit(1)
    }
}

fn tmpfile_to_stdout(tmpfile: &Path) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = File::open(tmpfile)?;
    io::copy(&mut reader, &mut stdout)?;
    Ok(())
}
