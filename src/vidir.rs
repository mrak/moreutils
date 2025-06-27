use rand::Rng;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Error, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{env, fs, io, process};

use crate::common;

fn usage() {
    eprintln!("Usage: vidir [--verbose] [DIRECTORY|FILE|-]...");
}

pub fn vidir() -> io::Result<()> {
    let (files, verbose) = entries_from_args()?;
    if files.is_empty() {
        process::exit(0);
    }

    let tmpfile = env::temp_dir().join(format!("vidir_{}.vidir", std::process::id()));
    let result = write_tmpfile(&files, &tmpfile)
        .and_then(|_| common::edit_tmpfile(&tmpfile))
        .and_then(|_| operate_tmpfile(&files, verbose, &tmpfile));

    match result {
        Err(e) if e.kind() == io::ErrorKind::Other => {
            eprintln!("{}", e);
            process::exit(1)
        }
        _ => result,
    }
}

fn write_tmpfile(files: &[PathBuf], tmpfile: &PathBuf) -> io::Result<()> {
    let format_width: usize = (files.len().ilog10() + 1).try_into().unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(tmpfile)?;
    for (i, pb) in files.iter().enumerate() {
        write!(file, "{i:0$} ", format_width)?;
        file.write_all(pb.as_os_str().as_bytes())?;
        writeln!(file)?;
    }
    Ok(())
}

fn entries_from_args() -> io::Result<(Vec<PathBuf>, bool)> {
    let mut unique_files: HashSet<PathBuf> = HashSet::new();
    let mut files: Vec<PathBuf> = Vec::new();
    let mut verbose = false;
    let mut double_dash = false;
    let mut sources: Vec<OsString> = Vec::new();

    for arg in env::args_os().skip(1) {
        if double_dash {
            sources.push(arg);
            continue;
        }
        match arg.to_str() {
            Some("--") => double_dash = true,
            Some("--verbose") => {
                verbose = true;
            }
            Some("--help" | "-h") => {
                usage();
                process::exit(0)
            }
            _ => sources.push(arg),
        }
    }

    if sources.is_empty() {
        sources.push(OsString::from("."));
    }

    for source in sources {
        match source.to_str() {
            Some("-") => {
                let stdin = io::stdin();
                let mut buffer = Vec::new();
                let mut reader = BufReader::new(stdin.lock());
                loop {
                    buffer.clear();
                    let bytes_read = reader.read_until(b'\n', &mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    let bytes = buffer
                        .last()
                        .map(|b| {
                            if *b == b'\n' {
                                &buffer[..buffer.len() - 1]
                            } else {
                                &buffer
                            }
                        })
                        .expect("buffer has at least one byte");
                    let pb = PathBuf::from(OsStr::from_bytes(bytes));
                    if (pb.is_file() || pb.is_symlink()) && unique_files.insert(pb.clone()) {
                        files.push(pb);
                    }
                }
            }
            _ => {
                let p = PathBuf::from(&source);
                if p.is_file() || p.is_symlink() {
                    if unique_files.insert(p.clone()) {
                        files.push(p);
                    }
                } else if p.is_dir() {
                    let entries: Vec<PathBuf> = fs::read_dir(p)?
                        .flatten()
                        .filter(|e| {
                            e.metadata()
                                .map(|m| m.is_file() || m.is_symlink())
                                .unwrap_or(false)
                        })
                        .map(|e| e.path())
                        .filter(|p| unique_files.insert(p.clone()))
                        .collect();
                    files.extend(entries);
                } else {
                    return Err(Error::other(format!(
                        "Cannot read file or directory: {source:?}"
                    )));
                }
            }
        }
    }

    Ok((files, verbose))
}

#[derive(Clone)]
struct Rename {
    temp_name: PathBuf,
    target_name: PathBuf,
}

fn operate_tmpfile(
    files: &[PathBuf],
    verbose: bool,
    tmpfile: &std::path::PathBuf,
) -> io::Result<()> {
    let format_width: usize = (files.len().ilog10() + 1).try_into().unwrap();
    let mut renames: Vec<Option<Rename>> = vec![None; files.len()];
    let mut buffer = Vec::new();
    let mut reader = BufReader::new(File::open(tmpfile)?);
    loop {
        buffer.clear();
        let bytes_read = reader.read_until(b'\n', &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let (numstr, mut rest) = buffer.split_at(format_width);
        rest = if rest[rest.len() - 1] == b'\n' {
            &rest[1..rest.len() - 1]
        } else {
            rest
        };
        let mut first_nonzero_i = 0;
        for (i, &b) in numstr.iter().enumerate() {
            if b != b'0' {
                first_nonzero_i = i;
                break;
            }
        }
        let num = str::from_utf8(&numstr[first_nonzero_i..])
            .map_err(Error::other)?
            .parse::<usize>()
            .map_err(Error::other)?;
        let target = PathBuf::from(OsStr::from_bytes(rest));
        let tmpname = temporary_filename(&files[num]);
        renames[num] = Some(Rename {
            temp_name: tmpname,
            target_name: target,
        });
    }

    for (i, o) in renames.iter().enumerate() {
        match o {
            Some(r) => {
                if verbose {
                    eprintln!("rename {:?} {:?}", &files[i], &r.temp_name);
                }
                std::fs::rename(&files[i], &r.temp_name)?;
            }
            None => {
                if verbose {
                    eprintln!("delete {:?}", &files[i]);
                }
                std::fs::remove_file(&files[i])?;
            }
        }
    }
    for r in renames.iter().flatten() {
        if verbose {
            eprintln!("rename {:?} {:?}", &r.temp_name, &r.target_name);
        }
        std::fs::rename(&r.temp_name, &r.target_name)?;
    }
    Ok(())
}

fn temporary_filename(source: &Path) -> PathBuf {
    let mut rng = rand::thread_rng();
    loop {
        let suffix = rng.gen_range(0..100000);
        let tmpname = source.with_extension(format!("{suffix:05}~"));
        if let Ok(false) = tmpname.try_exists() {
            return tmpname;
        }
    }
}
