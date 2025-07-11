use std::ffi::OsString;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::{env, process};

fn usage() {
    eprintln!("Usage: mispipe \"COMMAND\" \"COMMAND\"");
}

pub fn mispipe() -> io::Result<()> {
    let args = env::args_os().skip(1).take(2).collect::<Vec<OsString>>();
    let (cmd1, cmd2) = match &args[..] {
        [cmd1, cmd2] => (cmd1, cmd2),
        _ => {
            usage();
            process::exit(1);
        }
    };

    let mut child1 = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd1)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("failed to spawn {cmd1:?}"));
    let child1_stdout = child1.stdout.take().unwrap();
    let _ = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd2)
        .stdin(Stdio::from(child1_stdout))
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .map(|mut child2| child2.wait());

    if let Ok(status) = child1.wait() {
        match status.code() {
            Some(code) => process::exit(code),
            None => {
                // killed by signal
                let code = status.signal().map(|s| 128 + s).unwrap_or(1);
                process::exit(code);
            }
        }
    } else {
        eprintln!("Failed to execute command: {cmd1:?}");
        process::exit(2);
    }
}
