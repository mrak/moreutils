use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::process::exit;

fn usage() {
    eprintln!("Usage: ifne [-n] command");
}

pub fn ifne() -> io::Result<()> {
    let mut args = env::args().skip(1).peekable();
    let invert = match args.peek().map(|s| s.as_ref()) {
        Some("-n") => {
            let _ = args.next();
            true
        }
        None => {
            usage();
            exit(1)
        }
        _ => false,
    };

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut peek_buffer = [0; 1];
    let r = stdin.read(&mut peek_buffer);
    let has_content = matches!(r, Ok(1));

    if invert {
        // -n was passed.
        // if stdin is empty, execute program
        // if stdin is not empty, pass through to stdout
        if has_content {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(&peek_buffer)?;
            io::copy(&mut stdin, &mut stdout)?;
        } else {
            Command::new(args.next().unwrap())
                .args(args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
        }
    } else if has_content {
        // -n was NOT passed.
        // if stdin is not empty, execute program with stdin content
        // if stdin is empty, do nothing
        let mut cmd = Command::new(args.next().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut cmdstdin = cmd.stdin.take().expect("child stdin should be opened");
        cmdstdin.write_all(&peek_buffer)?;
        io::copy(&mut stdin, &mut cmdstdin)?;
    }

    Ok(())
}
