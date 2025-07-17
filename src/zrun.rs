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
    Bzip2,
    Xz,
    Lzma,
    Lzop,
    Zstd,
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
        cmd_args.push(
            match PathBuf::from(&arg).extension().and_then(OsStr::to_str) {
                Some("gz" | "Z") => decompress(Archive::Gzip, &arg),
                Some("bz2") => decompress(Archive::Bzip2, &arg),
                Some("xz") => decompress(Archive::Xz, &arg),
                Some("lzo") => decompress(Archive::Lzop, &arg),
                Some("lzma") => decompress(Archive::Lzma, &arg),
                Some("zst") => decompress(Archive::Zstd, &arg),
                _ => (None, arg),
            },
        );
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

fn decompress(archive_type: Archive, archive: &OsString) -> (Option<Decompressor>, OsString) {
    let pb = PathBuf::from(archive);
    let pb = pb.file_stem().expect("filename").to_owned();
    let prefix = OsString::from(format!("zrun_{}.", std::process::id()));
    let mut rand = 0;
    let mut tmpfilepath: PathBuf;
    loop {
        rand += 1;
        tmpfilepath = env::temp_dir();
        tmpfilepath = tmpfilepath.join(&prefix);
        tmpfilepath.push(rand.to_string());
        tmpfilepath.push(&pb);
        if let Ok(false) = tmpfilepath.try_exists() {
            break;
        }
    }
    let tmpfile = File::create(&tmpfilepath).expect("tmp directory to exist");
    let cmd = match archive_type {
        Archive::Gzip => "gzip",
        Archive::Bzip2 => "bzip2",
        Archive::Xz => "xz",
        Archive::Lzma => "lzma",
        Archive::Lzop => "lzop",
        Archive::Zstd => "zstd",
    };
    let child = Command::new(cmd)
        .arg("-d")
        .arg("-c")
        .arg(archive)
        .stdout(Stdio::from(tmpfile))
        .spawn()
        .expect("failed to execute command");
    (
        Some((Box::new(child), archive.clone())),
        tmpfilepath.as_os_str().to_owned(),
    )
}
