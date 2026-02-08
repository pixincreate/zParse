use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "zparse",
    version,
    about = "Parse and convert JSON/TOML/YAML/XML",
    after_help = "Examples:\n  zparse convert --from json --to toml input.json\n  zparse parse --from json input.json\n  cat input.xml | zparse parse --from xml"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse input and validate
    Parse(ParseArgs),
    /// Convert between formats
    Convert(ConvertArgs),
}

#[derive(Debug, Parser)]
struct ParseArgs {
    /// Input file (defaults to stdin)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    /// Input format (json, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    from: Option<FormatArg>,
    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
    /// Allow JSON comments (// and /* */)
    #[arg(long)]
    json_comments: bool,
    /// Allow trailing commas in JSON
    #[arg(long)]
    json_trailing_commas: bool,
}

#[derive(Debug, Parser)]
struct ConvertArgs {
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
    /// Allow JSON comments (// and /* */)
    #[arg(long)]
    json_comments: bool,
    /// Allow trailing commas in JSON
    #[arg(long)]
    json_trailing_commas: bool,
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
    match args.command {
        Command::Parse(parse_args) => run_parse(parse_args),
        Command::Convert(convert_args) => run_convert(convert_args),
    }
}

fn run_parse(args: ParseArgs) -> Result<()> {
    let input_data = read_input(&args.input)?;
    let from = match args.from.or_else(|| infer_format(&args.input)) {
        Some(format) => format,
        None => {
            bail!(
                "could not infer input format; pass --from or provide an input file with extension"
            );
        }
    };

    let json_config = zparse::JsonConfig::default()
        .with_comments(args.json_comments)
        .with_trailing_commas(args.json_trailing_commas);

    match from {
        FormatArg::Json => {
            let mut parser = zparse::json::Parser::with_config(input_data.as_bytes(), json_config);
            parser.parse_value()?;
        }
        FormatArg::Toml => {
            let mut parser = zparse::toml::Parser::new(input_data.as_bytes());
            parser.parse()?;
        }
        FormatArg::Yaml => {
            let mut parser = zparse::yaml::Parser::new(input_data.as_bytes());
            parser.parse()?;
        }
        FormatArg::Xml => {
            let mut parser = zparse::xml::Parser::new(input_data.as_bytes());
            parser.parse()?;
        }
    }

    write_output(&args.output, b"ok\n")?;
    Ok(())
}

fn run_convert(args: ConvertArgs) -> Result<()> {
    let input_data = read_input(&args.input)?;
    let from = match args.from.or_else(|| infer_format(&args.input)) {
        Some(format) => format,
        None => {
            bail!(
                "could not infer input format; pass --from or provide an input file with extension"
            );
        }
    };

    let json_config = zparse::JsonConfig::default()
        .with_comments(args.json_comments)
        .with_trailing_commas(args.json_trailing_commas);
    let options = zparse::ConvertOptions { json: json_config };
    let output = zparse::convert_with_options(&input_data, from.into(), args.to.into(), &options)?;

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
