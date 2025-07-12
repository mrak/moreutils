use std::{env, ffi::OsStr, io, os::unix::ffi::OsStrExt, process};

fn usage() {
    println!("Usage: zrun <command> <args>");
}

pub fn zrun() -> io::Result<()> {
    let mut args = env::args_os();
    let command = match args.next() {
        Some(os) => {
            if let Some("zrun") = os.to_str() {
                match args.next() {
                    Some(cmd) => cmd,
                    None => {
                        usage();
                        process::exit(1);
                    }
                }
            } else if let Some(b'z') = os.as_os_str().as_bytes().first() {
                OsStr::from_bytes(&os.as_os_str().as_bytes()[1..]).to_owned()
            } else {
                match args.next() {
                    Some(cmd) => cmd,
                    None => {
                        usage();
                        process::exit(1);
                    }
                }
            }
        }
        None => {
            usage();
            process::exit(1);
        }
    };

    println!("{}", command.display());

    usage();
    Ok(())
}
