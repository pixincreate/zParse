use std::env;
use std::process;
use zparse::{enums::FileType, util_functions};

fn main() {
    let args: Vec<String> = env::args().collect();
    let (file_type, contents) = read(args);

    println!("{:?}: {}", file_type, contents)
}

fn read(args: Vec<String>) -> (FileType, String) {
    if args.len() != 3 {
        eprintln!("Usage: cargo run -- <json / toml> <filename>");
        process::exit(1);
    }

    let file_type = match args[1].to_lowercase().as_str() {
        "json" => FileType::Json,
        "toml" => FileType::Toml,
        _ => {
            eprintln!("Invalid file type: {}", args[1]);
            process::exit(1);
        }
    };

    match util_functions::read_file(&args[2]) {
        Ok(contents) => (file_type, contents),
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(1);
        }
    }
}
