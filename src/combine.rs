use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, StdoutLock, Write};
use std::path::{Path, PathBuf};
use std::{env, process};

fn usage() {
    eprintln!("Usage: combine file1 OP file2");
}

enum Op {
    And,
    Not,
    Or,
    Xor,
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
    let file1 = PathBuf::from(&args.get(1).unwrap());
    let op_arg = args.get(2).unwrap();
    let file2 = PathBuf::from(&args.get(3).unwrap());

    let op = match op_arg.to_str() {
        Some("and") => Op::And,
        Some("not") => Op::Not,
        Some("or") => Op::Or,
        Some("xor") => Op::Xor,
        _ => {
            eprintln!("unknown operation, {op_arg:?}");
            process::exit(255);
        }
    };

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    match op {
        Op::And => op_and(&mut stdout, &file1, &file2),
        Op::Not => op_not(&mut stdout, &file1, &file2),
        Op::Or => op_or(&mut stdout, &file1, &file2),
        Op::Xor => op_xor(&mut stdout, &file1, &file2),
    }
}

fn op_and(stdout: &mut StdoutLock, file1: &Path, file2: &Path) -> io::Result<()> {
    let mut hs = HashSet::new();
    let mut buffer: Vec<u8> = Vec::new();

    let mut file1_reader = BufReader::new(File::open(file1)?);
    while let Ok(n) = file1_reader.read_until(b'\n', &mut buffer) {
        if n == 0 {
            break;
        }
        hs.insert(buffer.clone());
        buffer.clear();
    }

    buffer.clear();
    let mut file2_reader = BufReader::new(File::open(file2)?);
    while let Ok(n) = file2_reader.read_until(b'\n', &mut buffer) {
        if n == 0 {
            break;
        }
        if hs.contains(&buffer) {
            stdout.write_all(&buffer)?;
        }
        buffer.clear();
    }
    Ok(())
}
fn op_not(stdout: &mut StdoutLock, file1: &Path, file2: &Path) -> io::Result<()> {
    let mut hs = HashSet::new();
    let mut buffer: Vec<u8> = Vec::new();

    let mut file2_reader = BufReader::new(File::open(file2)?);
    while let Ok(n) = file2_reader.read_until(b'\n', &mut buffer) {
        if n == 0 {
            break;
        }
        hs.insert(buffer.clone());
        buffer.clear();
    }

    buffer.clear();
    let mut file1_reader = BufReader::new(File::open(file1)?);
    while let Ok(n) = file1_reader.read_until(b'\n', &mut buffer) {
        if n == 0 {
            break;
        }
        if !hs.contains(&buffer) {
            stdout.write_all(&buffer)?;
        }
        buffer.clear();
    }
    Ok(())
}
fn op_or(stdout: &mut StdoutLock, file1: &Path, file2: &Path) -> io::Result<()> {
    let mut f1 = File::open(file1)?;
    io::copy(&mut f1, stdout)?;
    let mut f2 = File::open(file2)?;
    io::copy(&mut f2, stdout)?;
    Ok(())
}
fn op_xor(stdout: &mut StdoutLock, file1: &Path, file2: &Path) -> io::Result<()> {
    let mut hs = HashSet::new();
    let mut buffer: Vec<u8> = Vec::new();

    let mut file1_reader = BufReader::new(File::open(file1)?);
    while let Ok(n) = file1_reader.read_until(b'\n', &mut buffer) {
        if n == 0 {
            break;
        }
        hs.insert(buffer.clone());
        buffer.clear();
    }

    buffer.clear();
    let mut file2_reader = BufReader::new(File::open(file2)?);
    while let Ok(n) = file2_reader.read_until(b'\n', &mut buffer) {
        if n == 0 {
            break;
        }
        hs.insert(buffer.clone());
        buffer.clear();
    }
    Ok(())
}
