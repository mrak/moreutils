use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let arg0 = env::args().next();
    let cmd = arg0
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .unwrap();
    match cmd {
        "sponge" => moarutils::sponge(),
        "vipe" => moarutils::vipe(),
        x => panic!("not implemented: {}", x),
    }
}
