use std::io;

fn usage() {
    println!("Usage: vidir [--verbose] [DIRECTORY|FILE|-]...");
}

pub fn vidir() -> io::Result<()> {
    usage();
    unimplemented!();
}
