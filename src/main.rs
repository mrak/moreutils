use std::env;
use std::process::exit;

struct Options {
    append: bool,
    file: String,
}

fn usage() {
    println!("sponge [-a] FILE");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = match args.len() {
        2 => Options {
            append: args[0].eq("-a"),
            file: args[1].clone(),
        },
        1 => Options {
            append: false,
            file: args[0].clone(),
        },
        _ => {
            usage();
            exit(1)
        }
    };
    println!("append: {}, file: {}", options.append, options.file);
}
