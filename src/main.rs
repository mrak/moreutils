use std::env;
use std::io;
use std::path::Path;

fn usage() {
    eprintln!(
        "
moreutils is a multi-program binary that changes its operation based on the name it is
invoked with. The following programs are included:

  chronic
  combine
  errno
  ifdata
  ifne
  isutf8
  mispipe
  pee
  parallel
  sponge
  ts
  vipe
  vidir
  zrun
"
    );
}

fn main() -> io::Result<()> {
    let arg0 = env::args_os().next();
    let cmd = arg0
        .as_deref()
        .map(Path::new)
        .and_then(Path::file_name)
        .expect("always at least the program name");
    match cmd.to_str() {
        Some("chronic") => moreutils::chronic(),
        Some("combine" | "_") => moreutils::combine(),
        Some("errno") => moreutils::errno(),
        Some("ifdata") => moreutils::ifdata(),
        Some("ifne") => moreutils::ifne(),
        Some("isutf8") => moreutils::isutf8(),
        Some("mispipe") => moreutils::mispipe(),
        Some("parallel") => moreutils::parallel(),
        Some("pee") => moreutils::pee(),
        Some("sponge") => moreutils::sponge(),
        Some("ts") => moreutils::ts(),
        Some("vipe") => moreutils::vipe(),
        Some("vidir") => moreutils::vidir(),
        Some(z) if z.starts_with("z") => moreutils::zrun(),
        Some("moreutils") => {
            usage();
            Ok(())
        }
        _ => {
            usage();
            eprintln!("not implemented: {cmd:?}");
            Ok(())
        }
    }
}
