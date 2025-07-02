use std::ffi::OsString;
use std::io;
use std::{env, process};

fn usage() {
    eprintln!("Usage: combine file1 OP file2");
}

pub fn combine() -> io::Result<()> {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() < 4 || args.len() > 5 {
        usage();
        process::exit(1);
    }
    if (args.len() == 5) && (args.first().unwrap() != "_" || args.last().unwrap() != "_") {
        usage();
        process::exit(1);
    }
    let file1 = args.get(1).unwrap();
    let op = args.get(2).unwrap();
    let file2 = args.get(3).unwrap();

    Ok(())
}
