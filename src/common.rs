use std::cmp::min;
use std::env;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{self, BufRead};
use std::os::unix::ffi::OsStringExt;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

pub fn get_editor() -> String {
    env::var("VISUAL")
        .map_err(|_| env::var("EDITOR"))
        .unwrap_or("vi".to_owned())
}

pub fn edit_tmpfile(tmpfile: &Path) -> io::Result<()> {
    let editor = get_editor();

    let tty_in = OpenOptions::new().read(true).open("/dev/tty")?;
    let tty_out = OpenOptions::new().write(true).open("/dev/tty")?;

    let status = Command::new(&editor)
        .arg(tmpfile)
        .stdin(Stdio::from(tty_in))
        .stdout(Stdio::from(tty_out))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{editor} exited nonzero, aborting",
        )))
    }
}

pub struct RingBuffer {
    head: usize,
    size: usize,
    capacity: usize,
    data: Vec<u8>,
}

impl RingBuffer {
    pub fn new(size: usize) -> RingBuffer {
        RingBuffer {
            head: 0,
            size: 0,
            capacity: size,
            data: vec![0; size],
        }
    }

    pub fn insert(&mut self, byte: u8) {
        self.size = min(self.capacity, self.size + 1);
        self.data[self.head] = byte;
        self.head = (self.head + 1) % self.capacity;
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::<u8>::with_capacity(self.size);
        let start = (self.head - self.size + self.capacity) % self.capacity;
        let mut i = 0;
        while i < self.size {
            vec.push(self.data[(start + i) % self.capacity]);
            i += 1;
        }
        vec
    }

    pub fn into_vec(mut self) -> Vec<u8> {
        if self.size == self.head {
            self.data[0..self.size].to_owned()
        } else {
            rotate_vector(&mut self.data, self.size - self.head);
            self.data
        }
    }
}

fn rotate_vector<T>(vec: &mut [T], n: usize) {
    let n = n % vec.len();
    if n == 0 {
        return;
    }
    vec.reverse();
    vec[0..n].reverse();
    vec[n..].reverse();
}

pub struct OsLines<B> {
    reader: B,
    buf: Vec<u8>,
}

impl<B: BufRead> Iterator for OsLines<B> {
    type Item = io::Result<OsString>;

    fn next(&mut self) -> Option<io::Result<OsString>> {
        self.buf.clear();

        match self.reader.read_until(b'\n', &mut self.buf) {
            Ok(0) => None,
            Ok(_) => {
                if self.buf.ends_with(b"\n") {
                    self.buf.pop();
                    if self.buf.ends_with(b"\r") {
                        self.buf.pop();
                    }
                }
                Some(Ok(OsString::from_vec(self.buf.clone())))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

pub trait OsLinesExt: BufRead {
    fn os_lines(self) -> OsLines<Self>
    where
        Self: Sized;
}

impl<R: BufRead> OsLinesExt for R {
    fn os_lines(self) -> OsLines<Self>
    where
        Self: Sized,
    {
        OsLines {
            reader: self,
            buf: Vec::new(),
        }
    }
}
