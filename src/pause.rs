use std::io;
use std::thread;
use std::time;

pub fn pause() -> io::Result<()> {
    thread::sleep(time::Duration::MAX);
    Ok(())
}
