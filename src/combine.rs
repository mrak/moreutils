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

struct Args {
    file1: PathBuf,
    op: Op,
    file2: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() < 4 || args.len() > 5 {
        return Err(String::from("unexpected number of arguments"));
    }
    if (args.len() == 5) && (args.first().unwrap() != "_" || args.last().unwrap() != "_") {
        return Err(String::from("unexpected number of arguments"));
    }
    let file1 = PathBuf::from(&args.get(1).unwrap());
    let op = match args.get(2).unwrap().to_str() {
        Some("and") => Op::And,
        Some("not") => Op::Not,
        Some("or") => Op::Or,
        Some("xor") => Op::Xor,
        _ => return Err(String::from("unknown operation, {op_arg:?}")),
    };
    let file2 = PathBuf::from(&args.get(3).unwrap());
    Ok(Args { file1, op, file2 })
}

pub fn combine() -> io::Result<()> {
    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        usage();
        process::exit(1);
    });

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    match args.op {
        Op::And => op_and(&mut stdout, &args.file1, &args.file2),
        Op::Not => op_not(&mut stdout, &args.file1, &args.file2),
        Op::Or => op_or(&mut stdout, &args.file1, &args.file2),
        Op::Xor => op_xor(&mut stdout, &args.file1, &args.file2),
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
        if !hs.remove(&buffer) {
            stdout.write_all(&buffer)?;
        }
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
