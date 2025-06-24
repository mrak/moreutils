use std::{env, io};

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
        _ => false,
    };
    println!("invert is {}", invert);
    Ok(())
}
