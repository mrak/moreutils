use std::env;
use std::io;
use std::path::Path;

fn usage() {
    eprintln!(
        "
moreutils is a multi-program binary that changes its operation based on the name it is
invoked with. The following programs are included:

  errno
  ifne
  pee
  sponge
  ts
  vipe
  vidir
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
        Some("combine" | "_") => moreutils::combine(),
        Some("errno") => moreutils::errno(),
        Some("ifne") => moreutils::ifne(),
        Some("pee") => moreutils::pee(),
        Some("sponge") => moreutils::sponge(),
        Some("ts") => moreutils::ts(),
        Some("vipe") => moreutils::vipe(),
        Some("vidir") => moreutils::vidir(),
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
