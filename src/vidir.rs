use std::io;

fn usage() {
    eprintln!("Usage: vidir [--verbose] [DIRECTORY|FILE|-]...");
}

pub fn vidir() -> io::Result<()> {
    usage();
    unimplemented!();
}
