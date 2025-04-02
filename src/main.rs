use moarutils::sponge;
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
        "sponge" => sponge::sponge(),
        x => panic!("not implemented: {}", x),
    }
}
