use clap::Parser;
use std::{error::Error, fs::File, io::Write};
use zparse::util_functions::{read, Args};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let response = read(args)?;

    if let Some(output_path) = response.output {
        let mut file = File::create(output_path)?;
        file.write_all(response.contents.as_bytes())?;
    } else {
        println!("{}", response.contents);
    }

    Ok(())
}
