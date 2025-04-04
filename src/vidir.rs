use std::io;

fn usage() {
    println!("Usage: vidir [--verbose] [directory|file|-]...");
}

pub fn vidir() -> io::Result<()> {
    usage();
    Ok(())
}
