use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "zparse",
    version,
    about = "Parse and convert JSON/TOML/YAML/XML"
)]
struct Args {
    /// Input file (defaults to stdin)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    /// Input format (json, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    from: Option<FormatArg>,
    /// Output format (json, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    to: FormatArg,
    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

#[derive(Clone, Debug, ValueEnum)]
enum FormatArg {
    Json,
    Toml,
    #[value(alias = "yml")]
    Yaml,
    Xml,
}

impl From<FormatArg> for zparse::Format {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Json => zparse::Format::Json,
            FormatArg::Toml => zparse::Format::Toml,
            FormatArg::Yaml => zparse::Format::Yaml,
            FormatArg::Xml => zparse::Format::Xml,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_data = read_input(&args.input)?;
    let from = match args.from.or_else(|| infer_format(&args.input)) {
        Some(format) => format,
        None => {
            bail!(
                "could not infer input format; pass --from or provide an input file with extension"
            );
        }
    };

    let output = zparse::convert(&input_data, from.into(), args.to.into())?;

    write_output(&args.output, output.as_bytes())?;
    Ok(())
}

fn read_input(path: &Option<PathBuf>) -> Result<String> {
    match path {
        Some(path) => std::fs::read_to_string(path)
            .with_context(|| format!("failed to read input file {}", path.display())),
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("failed to read stdin")?;
            if buffer.trim().is_empty() {
                bail!("no input provided on stdin");
            }
            Ok(buffer)
        }
    }
}

fn write_output(path: &Option<PathBuf>, data: &[u8]) -> Result<()> {
    match path {
        Some(path) => std::fs::write(path, data)
            .with_context(|| format!("failed to write output file {}", path.display())),
        None => {
            let mut stdout = io::stdout();
            stdout.write_all(data).context("failed to write stdout")?;
            Ok(())
        }
    }
}

fn infer_format(path: &Option<PathBuf>) -> Option<FormatArg> {
    let path = path.as_ref()?;
    let ext = path.extension().and_then(|s| s.to_str())?;
    match ext {
        "json" => Some(FormatArg::Json),
        "toml" => Some(FormatArg::Toml),
        "yaml" | "yml" => Some(FormatArg::Yaml),
        "xml" => Some(FormatArg::Xml),
        _ => None,
    }
}

#[allow(dead_code)]
fn is_stdin(path: &Option<PathBuf>) -> bool {
    path.as_ref().map(Path::new).is_none()
}
