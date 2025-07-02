use std::env;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

pub fn get_editor() -> String {
    env::var("VISUAL")
        .map_err(|_| env::var("EDITOR"))
        .unwrap_or("vi".to_owned())
}

pub fn edit_tmpfile(tmpfile: &Path) -> io::Result<()> {
    let editor = get_editor();

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
        Err(io::Error::other(format!(
            "{editor} exited nonzero, aborting",
        )))
    }
}
