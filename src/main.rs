use std::env;
use zparse::util_functions::read;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (file_type, contents) = read(args).unwrap_or_else(|error| {
        eprintln!("{}", error);
        std::process::exit(1);
    });

    println!("{:?}\n{}", file_type, contents)
}
