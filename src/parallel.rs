use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, exit};
use std::thread;
use std::time;

use signal_hook::consts::SIGCHLD;
use signal_hook::iterator::SignalsInfo;
use signal_hook::iterator::exfiltrator::origin::WithOrigin;
use sysinfo::System;

fn usage() {
    eprintln!("parallel [OPTIONS] command -- arguments");
    eprintln!("        for each argument, run command iwth argument, in parallel");
    eprintln!("parallel [OPTIONS] -- commands");
    eprintln!("        run specified commands in parallel");
}

#[derive(Debug)]
struct Execution {
    command: OsString,
    args: Vec<OsString>,
}

pub fn parallel() -> io::Result<()> {
    let mut interpolate = false;
    let mut n_args: usize = 1;
    let mut maxload: Option<f64> = None;
    let mut maxjobs = thread::available_parallelism().map_or(1, |n| n.get());

    let mut args = env::args_os().skip(1).peekable();

    while let Some(arg) = args.peek() {
        if arg == "--" || arg.as_bytes().first().is_none_or(|b| *b != b'-') {
            break;
        }

        let arg = args.next().unwrap(); // consume peeked arg
        match arg.to_str() {
            Some("-h") => {
                usage();
                exit(0);
            }
            Some("-i") => interpolate = true,
            Some("-n") => {
                n_args = args
                    .next()
                    .and_then(|os_str| os_str.to_str().and_then(|s| s.parse::<usize>().ok()))
                    .unwrap_or_else(|| {
                        eprintln!("parallel: -n requires a positive integer argument");
                        exit(1);
                    })
            }
            Some("-l") => {
                maxload = Some(
                    args.next()
                        .and_then(|os_str| os_str.to_str().and_then(|s| s.parse::<f64>().ok()))
                        .unwrap_or_else(|| {
                            eprintln!("parallel: -l requires a number argument");
                            exit(1);
                        }),
                )
            }
            Some("-j") => {
                maxjobs = args
                    .next()
                    .and_then(|os_str| os_str.to_str().and_then(|s| s.parse::<usize>().ok()))
                    .unwrap_or_else(|| {
                        eprintln!("parallel: -j requires a positive integer argument");
                        exit(1);
                    })
            }
            _ => {
                eprintln!("parallel: invalid option -- '{}'", arg.display());
                usage();
                exit(1);
            }
        }
    }

    let jobs: Vec<Execution> = if let Some("--") = args.peek().and_then(|a| a.to_str()) {
        args.skip(1)
            .map(|a| Execution {
                command: OsString::from("sh"),
                args: vec![OsString::from("-c"), a],
            })
            .collect()
    } else {
        let command = args.next().unwrap();
        let (fixed_args, parallel_args) = split_args(args);
        parallel_args
            .chunks(n_args)
            .map(|chunk| {
                let mut fa = fixed_args.clone();
                fa.extend_from_slice(chunk);
                Execution {
                    command: command.clone(),
                    args: fa,
                }
            })
            .collect()
    };

    println!("-i {interpolate} -l {maxload:?} -j {maxjobs:?} -n {n_args}");
    for e in jobs.iter() {
        println!("{e:?}");
    }

    exit(pool_jobs(maxjobs, maxload, jobs)?);
}

fn pool_jobs(maxjobs: usize, maxload: Option<f64>, jobs: Vec<Execution>) -> io::Result<i32> {
    let mut exit_code = 0;
    let mut jobs_running: HashMap<u32, Child> = HashMap::new();
    let mut binding = SignalsInfo::<WithOrigin>::new([SIGCHLD])?;
    let mut signals = binding.forever();
    for job in jobs {
        if jobs_running.len() == maxjobs {
            match signals.next() {
                Some(origin) if origin.signal == SIGCHLD => {
                    let pid = origin.process.unwrap().pid;
                    let mut child = jobs_running.remove(&(pid as u32)).unwrap();
                    let status = child.wait()?;
                    exit_code |= match status.code() {
                        Some(code) => code,
                        None => status.signal().map_or_else(|| 1, |sig| 128 + sig),
                    }
                }
                _ => unreachable!(),
            }
        }

        if let Some(maxload) = maxload {
            wait_for_load(maxload);
        }

        let child = Command::new(&job.command).args(&job.args).spawn()?;
        jobs_running.insert(child.id(), child);
    }
    for (_, mut child) in jobs_running {
        exit_code |= child.wait()?.code().unwrap_or(1);
    }
    Ok(exit_code)
}

fn wait_for_load(maxload: f64) {
    loop {
        if get_load_average() < maxload {
            break;
        }
        thread::sleep(time::Duration::from_millis(500));
    }
}

fn split_args<I>(iter: I) -> (Vec<OsString>, Vec<OsString>)
where
    I: Iterator<Item = OsString>,
{
    let mut before: Vec<OsString> = Vec::new();
    let mut after: Vec<OsString> = Vec::new();
    let mut double_dash = false;

    for item in iter {
        if double_dash {
            after.push(item);
        } else if item == "--" {
            double_dash = true;
        } else {
            before.push(item);
        }
    }

    (before, after)
}

fn get_load_average() -> f64 {
    System::load_average().one
}
