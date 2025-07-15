use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, File},
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::{self, Child, Command, Stdio},
};

fn usage() {
    println!("Usage: zrun <command> <args>");
}

enum Archive {
    Gzip,
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

    let mut decompressors: Vec<(Option<&mut Child>, &OsStr)> = Vec::new();
    let cmd_args: Vec<OsString> = args
        .map(|arg| {
            if arg.as_bytes().ends_with(b".gz") || arg.as_bytes().ends_with(b".Z") {
                let pb = PathBuf::from(&arg);
                let pb = pb.file_stem().expect("TODO").to_owned();
                let mut rand = 0;
                loop {
                    rand += 1;
                    let mut tmpfilename = OsString::from(format!("zrun_{}.", std::process::id()));
                    tmpfilename.push(rand.to_string());
                    tmpfilename.push(&pb);
                    let tmpfilepath = PathBuf::from(&tmpfilename);
                    if let Ok(false) = tmpfilepath.try_exists() {
                        let child = decompress(Archive::Gzip, arg, &tmpfilepath);
                        decompressors.push((Some(&mut child), &tmpfilename));
                        return tmpfilename;
                    }
                }
            } else {
                arg
            }
        })
        .collect();

    for (child, file) in decompressors.iter_mut() {
        match child.wait() {
            Ok(status) => match status.code() {
                Some(0) => {}
                _ => {
                    eprintln!("zrun: failed to decompress file: {file:?}");
                    process::exit(1);
                }
            },
            Err(_) => {
                eprintln!("zrun: failed to decompress file: {file:?}");
                process::exit(1);
            }
        }
    }

    println!("{} {:?}", command.display(), cmd_args);
    for (_, f) in decompressors {
        fs::remove_file(f)?;
    }
    Ok(())
}

fn decompress(archive_type: Archive, archive: OsString, tmpfilepath: &PathBuf) -> Child {
    let tmpfile = File::create(tmpfilepath).expect("TODO");
    match archive_type {
        Archive::Gzip => Command::new("gzip")
            .arg("-d")
            .arg("-c")
            .arg(archive)
            .stdout(Stdio::from(tmpfile))
            .spawn()
            .expect("TODO"),
    }
}
