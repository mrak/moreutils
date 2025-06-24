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
        .expect("command name should be UTF-8 compliant");
    match cmd {
        "ifne" => moreutils::ifne(),
        "sponge" => moreutils::sponge(),
        "ts" => moreutils::ts(),
        "vipe" => moreutils::vipe(),
        "vidir" => moreutils::vidir(),
        x => panic!("not implemented: {}", x),
    }
}
