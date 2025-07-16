use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, File},
    io,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    process::{self, Child, Command, Stdio},
};

fn usage() {
    println!("Usage: zrun <command> <args>");
}

type Decompressor = (Box<Child>, OsString);

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

    let mut cmd_args: Vec<(Option<Decompressor>, OsString)> = Vec::new();
    for arg in args {
        if arg.as_bytes().ends_with(b".gz") || arg.as_bytes().ends_with(b".Z") {
            let pb = PathBuf::from(&arg);
            let pb = pb.file_stem().expect("TODO").to_owned();
            let mut rand = 0;
            let mut tmpfilename: OsString;
            let mut tmpfilepath: PathBuf;
            loop {
                rand += 1;
                tmpfilename = OsString::from(format!("zrun_{}.", std::process::id()));
                tmpfilename.push(rand.to_string());
                tmpfilename.push(&pb);
                tmpfilepath = PathBuf::from(&tmpfilename);
                if let Ok(false) = tmpfilepath.try_exists() {
                    break;
                }
            }
            let child: Box<Child> = decompress(Archive::Gzip, &arg, &tmpfilepath);
            cmd_args.push((Some((child, arg)), tmpfilename));
        } else {
            cmd_args.push((None, arg));
        }
    }

    for (decompressor, _) in cmd_args.iter_mut() {
        if let Some((child, file)) = decompressor {
            match child.wait().ok().and_then(|s| s.code()) {
                Some(0) => {}
                _ => {
                    eprintln!("zrun: failed to decompress file: {file:?}");
                    process::exit(1);
                }
            }
        }
    }

    let child = Command::new(&command)
        .args(cmd_args.iter().map(|(_, arg)| arg))
        .spawn();

    let exit_code = match child {
        Ok(mut child) => match child.wait() {
            Ok(status) => status.code().unwrap_or(1),
            Err(_) => 1,
        },
        Err(e) => {
            eprintln!("zrun: failed to execute command: {}", command.display());
            eprintln!("{e}");
            1
        }
    };

    for (decompressor, file) in cmd_args {
        if decompressor.is_some() {
            fs::remove_file(file)?;
        }
    }
    process::exit(exit_code);
}

fn decompress(archive_type: Archive, archive: &OsString, tmpfilepath: &PathBuf) -> Box<Child> {
    let tmpfile = File::create(tmpfilepath).expect("TODO");
    match archive_type {
        Archive::Gzip => Box::new(
            Command::new("gzip")
                .arg("-d")
                .arg("-c")
                .arg(archive)
                .stdout(Stdio::from(tmpfile))
                .spawn()
                .expect("TODO"),
        ),
    }
}
