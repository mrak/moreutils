use std::env;
use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command, exit};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

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

    let exit_code = if jobs.len() <= maxjobs && maxload.is_none() {
        spawn_all(jobs)?
    } else {
        pool_jobs(maxjobs, maxload, jobs)?
    };

    exit(exit_code);
}

fn pool_jobs(maxjobs: usize, maxload: Option<f64>, jobs: Vec<Execution>) -> io::Result<i32> {
    let (job_tx, job_rx): (mpsc::Sender<Execution>, mpsc::Receiver<Execution>) = mpsc::channel();
    let (code_tx, code_rx): (mpsc::Sender<i32>, mpsc::Receiver<i32>) = mpsc::channel();
    let job_rx_arc = Arc::new(Mutex::new(job_rx));
    for i in 0..maxjobs {
        let blah = i;
        let rx = Arc::clone(&job_rx_arc);
        let tx = code_tx.clone();
        thread::spawn(move || {
            while let Ok(e) = rx.lock().unwrap().recv() {
                println!("executing from pool {blah}");
                match Command::new(&e.command).args(&e.args).status() {
                    Ok(status) => {
                        let _ = tx.send(status.code().unwrap_or(1));
                    }
                    Err(_) => {
                        let _ = tx.send(1);
                    }
                };
            }
        });
    }

    let n = jobs.len();
    let mut exit_code = 0;

    for job in jobs {
        let _ = job_tx.send(job);
    }

    for _ in 0..n {
        match code_rx.recv() {
            Ok(code) => exit_code |= code,
            Err(_) => todo!(),
        }
    }
    Ok(exit_code)
}

fn spawn_all(jobs: Vec<Execution>) -> io::Result<i32> {
    let children = jobs
        .iter()
        .map(|e| Command::new(&e.command).args(&e.args).spawn())
        .collect::<io::Result<Vec<Child>>>()?;

    children
        .into_iter()
        .try_fold(0, |acc, mut child| match child.wait() {
            Ok(status) => match status.code() {
                Some(c) => Ok(acc | c),
                None => Ok(acc | status.signal().map_or_else(|| 1, |sig| 128 + sig)),
            },
            Err(e) => Err(e),
        })
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
