use regex::Regex;
use std::{env, io, process};

pub fn errno() -> io::Result<()> {
    let re = Regex::new(r" \(os error \d+\)").expect("compile static regex");
    let errnos: Vec<Errno> = vec![
        // BEGIN ERRNOS
        Errno {
            name: "EUCLEAN",
            id: 117,
        },
        Errno {
            name: "ESTALE",
            id: 116,
        },
        Errno {
            name: "ETOOMANYREFS",
            id: 109,
        },
        Errno {
            name: "EADDRINUSE",
            id: 98,
        },
        Errno {
            name: "EIDRM",
            id: 43,
        },
        Errno {
            name: "ELOOP",
            id: 40,
        },
        Errno {
            name: "ERANGE",
            id: 34,
        },
        Errno {
            name: "EINVAL",
            id: 22,
        },
        Errno {
            name: "EEXIST",
            id: 17,
        },
        Errno {
            name: "ENOMEM",
            id: 12,
        },
        Errno {
            name: "EAGAIN",
            id: 11,
        },
        Errno {
            name: "ESRCH",
            id: 3,
        },
        Errno {
            name: "ENOTRECOVERABLE",
            id: 131,
        },
        Errno {
            name: "EISNAM",
            id: 120,
        },
        Errno {
            name: "EHOSTUNREACH",
            id: 113,
        },
        Errno {
            name: "EPROTOTYPE",
            id: 91,
        },
        Errno {
            name: "ESTRPIPE",
            id: 86,
        },
        Errno {
            name: "ELIBACC",
            id: 79,
        },
        Errno {
            name: "EREMCHG",
            id: 78,
        },
        Errno {
            name: "EXFULL",
            id: 54,
        },
        Errno {
            name: "EL3HLT",
            id: 46,
        },
        Errno {
            name: "ENAMETOOLONG",
            id: 36,
        },
        Errno {
            name: "ENOTSUP",
            id: 95,
        },
        Errno {
            name: "EKEYREVOKED",
            id: 128,
        },
        Errno {
            name: "ENOKEY",
            id: 126,
        },
        Errno {
            name: "EDQUOT",
            id: 122,
        },
        Errno {
            name: "EAFNOSUPPORT",
            id: 97,
        },
        Errno {
            name: "ELIBMAX",
            id: 82,
        },
        Errno {
            name: "EADV",
            id: 68,
        },
        Errno {
            name: "ENOPKG",
            id: 65,
        },
        Errno {
            name: "EUNATCH",
            id: 49,
        },
        Errno {
            name: "EWOULDBLOCK",
            id: 11,
        },
        Errno {
            name: "EDEADLK",
            id: 35,
        },
        Errno {
            name: "EBUSY",
            id: 16,
        },
        Errno {
            name: "ENOENT",
            id: 2,
        },
        Errno {
            name: "ERFKILL",
            id: 132,
        },
        Errno {
            name: "EINPROGRESS",
            id: 115,
        },
        Errno {
            name: "ECONNREFUSED",
            id: 111,
        },
        Errno {
            name: "ETIMEDOUT",
            id: 110,
        },
        Errno {
            name: "ENOTCONN",
            id: 107,
        },
        Errno {
            name: "ECONNRESET",
            id: 104,
        },
        Errno {
            name: "ERESTART",
            id: 85,
        },
        Errno {
            name: "EILSEQ",
            id: 84,
        },
        Errno {
            name: "ETIME",
            id: 62,
        },
        Errno {
            name: "EBADRQC",
            id: 56,
        },
        Errno {
            name: "ENOANO",
            id: 55,
        },
        Errno {
            name: "EISDIR",
            id: 21,
        },
        Errno {
            name: "ENODEV",
            id: 19,
        },
        Errno {
            name: "ENXIO",
            id: 6,
        },
        Errno { name: "EIO", id: 5 },
        Errno {
            name: "EKEYREJECTED",
            id: 129,
        },
        Errno {
            name: "EHOSTDOWN",
            id: 112,
        },
        Errno {
            name: "ESHUTDOWN",
            id: 108,
        },
        Errno {
            name: "EISCONN",
            id: 106,
        },
        Errno {
            name: "ENOBUFS",
            id: 105,
        },
        Errno {
            name: "ENETUNREACH",
            id: 101,
        },
        Errno {
            name: "ENETDOWN",
            id: 100,
        },
        Errno {
            name: "EOVERFLOW",
            id: 75,
        },
        Errno {
            name: "EMULTIHOP",
            id: 72,
        },
        Errno {
            name: "EDEADLOCK",
            id: 35,
        },
        Errno {
            name: "ENOSYS",
            id: 38,
        },
        Errno {
            name: "ENOLCK",
            id: 37,
        },
        Errno {
            name: "ETXTBSY",
            id: 26,
        },
        Errno {
            name: "E2BIG",
            id: 7,
        },
        Errno {
            name: "EALREADY",
            id: 114,
        },
        Errno {
            name: "ELIBBAD",
            id: 80,
        },
        Errno {
            name: "EBADFD",
            id: 77,
        },
        Errno {
            name: "ENOTUNIQ",
            id: 76,
        },
        Errno {
            name: "ENOTEMPTY",
            id: 39,
        },
        Errno {
            name: "ENOSPC",
            id: 28,
        },
        Errno {
            name: "ENFILE",
            id: 23,
        },
        Errno {
            name: "EINTR",
            id: 4,
        },
        Errno {
            name: "ENOTNAM",
            id: 118,
        },
        Errno {
            name: "ENETRESET",
            id: 102,
        },
        Errno {
            name: "EPROTONOSUPPORT",
            id: 93,
        },
        Errno {
            name: "EDOTDOT",
            id: 73,
        },
        Errno {
            name: "EREMOTE",
            id: 66,
        },
        Errno {
            name: "EL3RST",
            id: 47,
        },
        Errno {
            name: "ECHRNG",
            id: 44,
        },
        Errno {
            name: "EDOM",
            id: 33,
        },
        Errno {
            name: "ESPIPE",
            id: 29,
        },
        Errno {
            name: "EKEYEXPIRED",
            id: 127,
        },
        Errno {
            name: "EMEDIUMTYPE",
            id: 124,
        },
        Errno {
            name: "EPFNOSUPPORT",
            id: 96,
        },
        Errno {
            name: "ENOTSOCK",
            id: 88,
        },
        Errno {
            name: "ELIBEXEC",
            id: 83,
        },
        Errno {
            name: "EBFONT",
            id: 59,
        },
        Errno {
            name: "EBADSLT",
            id: 57,
        },
        Errno {
            name: "EPIPE",
            id: 32,
        },
        Errno {
            name: "ENOTBLK",
            id: 15,
        },
        Errno {
            name: "ENOPROTOOPT",
            id: 92,
        },
        Errno {
            name: "ECOMM",
            id: 70,
        },
        Errno {
            name: "ESRMNT",
            id: 69,
        },
        Errno {
            name: "ENODATA",
            id: 61,
        },
        Errno {
            name: "ENOMSG",
            id: 42,
        },
        Errno {
            name: "EFBIG",
            id: 27,
        },
        Errno {
            name: "ENOTDIR",
            id: 20,
        },
        Errno {
            name: "ECHILD",
            id: 10,
        },
        Errno {
            name: "EREMOTEIO",
            id: 121,
        },
        Errno {
            name: "ECONNABORTED",
            id: 103,
        },
        Errno {
            name: "EADDRNOTAVAIL",
            id: 99,
        },
        Errno {
            name: "EHWPOISON",
            id: 133,
        },
        Errno {
            name: "EOWNERDEAD",
            id: 130,
        },
        Errno {
            name: "ESOCKTNOSUPPORT",
            id: 94,
        },
        Errno {
            name: "EDESTADDRREQ",
            id: 89,
        },
        Errno {
            name: "EBADMSG",
            id: 74,
        },
        Errno {
            name: "EPROTO",
            id: 71,
        },
        Errno {
            name: "ENOSR",
            id: 63,
        },
        Errno {
            name: "EBADR",
            id: 53,
        },
        Errno {
            name: "EBADE",
            id: 52,
        },
        Errno {
            name: "ELNRNG",
            id: 48,
        },
        Errno {
            name: "EL2NSYNC",
            id: 45,
        },
        Errno {
            name: "EMLINK",
            id: 31,
        },
        Errno {
            name: "EROFS",
            id: 30,
        },
        Errno {
            name: "ENOTTY",
            id: 25,
        },
        Errno {
            name: "EFAULT",
            id: 14,
        },
        Errno {
            name: "EPERM",
            id: 1,
        },
        Errno {
            name: "EOPNOTSUPP",
            id: 95,
        },
        Errno {
            name: "EMSGSIZE",
            id: 90,
        },
        Errno {
            name: "EUSERS",
            id: 87,
        },
        Errno {
            name: "ELIBSCN",
            id: 81,
        },
        Errno {
            name: "ENOLINK",
            id: 67,
        },
        Errno {
            name: "ENOSTR",
            id: 60,
        },
        Errno {
            name: "EBADF",
            id: 9,
        },
        Errno {
            name: "ECANCELED",
            id: 125,
        },
        Errno {
            name: "ENOMEDIUM",
            id: 123,
        },
        Errno {
            name: "ENAVAIL",
            id: 119,
        },
        Errno {
            name: "ENONET",
            id: 64,
        },
        Errno {
            name: "EL2HLT",
            id: 51,
        },
        Errno {
            name: "ENOCSI",
            id: 50,
        },
        Errno {
            name: "EMFILE",
            id: 24,
        },
        Errno {
            name: "EXDEV",
            id: 18,
        },
        Errno {
            name: "EACCES",
            id: 13,
        },
        Errno {
            name: "ENOEXEC",
            id: 8,
        },
        // END ERRNOS
    ];
    let mut args = env::args().skip(1).peekable();
    let mode = match args.peek().map(|a| a.as_ref()) {
        Some("-l") => Mode::List,
        Some("-s") => Mode::Search(String::from("ENOENT")),
        Some("-S") => Mode::SearchAllLocale(String::from("ENOENT")),
        Some(a) => {
            if let Ok(i) = a.parse::<i32>() {
                Mode::LookupCode(i)
            } else {
                Mode::LookupName(String::from(a))
            }
        }
        None => {
            process::exit(0);
        }
    };

    match mode {
        Mode::List => {
            for errno in errnos {
                let description = std::io::Error::from_raw_os_error(errno.id).to_string();
                let description = re.replace(&description, "");
                println!("{} {} {}", errno.name, errno.id, description,);
            }
        }
        Mode::LookupName(s) => match errnos.iter().find(|e| e.name == s) {
            Some(errno) => {
                print_errno(errno);
            }
            None => {
                println!("Unknown errno");
            }
        },
        Mode::LookupCode(c) => match errnos.iter().find(|e| e.id == c) {
            Some(errno) => {
                print_errno(errno);
            }
            None => {
                println!("Unknown errno");
            }
        },
        Mode::Search(s) => match errnos.iter().find(|&e| {
            std::io::Error::from_raw_os_error(e.id)
                .to_string()
                .contains(&s)
        }) {
            Some(errno) => {
                print_errno(errno);
            }
            None => {
                println!("Unknown errno");
            }
        },
        Mode::SearchAllLocale(_) => unimplemented!(),
        _ => unimplemented!(),
    }

    Ok(())
}

fn print_errno(errno: &Errno) {
    let re = Regex::new(r" \(os error \d+\)").unwrap();
    let description = std::io::Error::from_raw_os_error(errno.id).to_string();
    let description = re.replace(&description, "");
    println!("{} {} {}", errno.name, errno.id, description,);
}

enum Mode {
    LookupName(String),
    LookupCode(i32),
    List,
    Search(String),
    SearchAllLocale(String),
}

pub struct Errno<'a> {
    name: &'a str,
    id: i32,
}
