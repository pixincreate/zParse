use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "zparse",
    version,
    about = "Parse and convert JSON/TOML/YAML/XML",
    after_help = "Examples:\n  zparse --parse input.json --print-output\n  zparse --convert input.json --from json --to toml\n  zparse convert --from json --to toml input.json\n  zparse parse --from json input.json\n  cat input.xml | zparse parse --from xml"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    /// Parse input and validate (top-level mode)
    #[arg(long, value_name = "INPUT", num_args = 0..=1, default_missing_value = "-", conflicts_with_all = ["command", "convert"])]
    parse: Option<PathBuf>,
    /// Convert between formats (top-level mode)
    #[arg(long, value_name = "INPUT", num_args = 0..=1, default_missing_value = "-", conflicts_with_all = ["command", "parse"])]
    convert: Option<PathBuf>,
    /// Input format (json, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    from: Option<FormatArg>,
    /// Output format (json, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    to: Option<FormatArg>,
    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
    /// Print the input content or converted output on success
    #[arg(long = "print-output")]
    print_output: bool,
    /// Allow JSON comments (// and /* */)
    #[arg(long)]
    json_comments: bool,
    /// Allow trailing commas in JSON
    #[arg(long)]
    json_trailing_commas: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse input and validate
    Parse(ParseArgs),
    /// Convert between formats
    Convert(ConvertArgs),
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
    /// Print the input content on successful parse
    #[arg(long = "print-output")]
    print_output: bool,
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
    /// Print the converted output on successful conversion
    #[arg(long = "print-output")]
    print_output: bool,
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

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(command) = args.command {
        return match command {
            Command::Parse(parse_args) => run_parse(parse_args),
            Command::Convert(convert_args) => run_convert(convert_args),
        };
    }

    if args.parse.is_some() {
        let parse_args = ParseArgs {
            input: normalize_flag_input(args.parse),
            from: args.from,
            output: args.output,
            print_output: args.print_output,
            json_comments: args.json_comments,
            json_trailing_commas: args.json_trailing_commas,
        };
        return run_parse(parse_args);
    }

    if args.convert.is_some() {
        let to = args
            .to
            .ok_or_else(|| anyhow::anyhow!("--to is required when using --convert"))?;
        let convert_args = ConvertArgs {
            input: normalize_flag_input(args.convert),
            from: args.from,
            to,
            output: args.output,
            print_output: args.print_output,
            json_comments: args.json_comments,
            json_trailing_commas: args.json_trailing_commas,
        };
        return run_convert(convert_args);
    }

    bail!("no command specified; use a subcommand or --parse/--convert");
}

fn run_parse(args: ParseArgs) -> Result<()> {
    let input_data = read_input(&args.input)?;
    let from = resolve_format(args.from, &args.input)?;
    let json_config = json_config_from_flags(args.json_comments, args.json_trailing_commas);

    match from {
        zparse::Format::Json => {
            let mut parser = zparse::json::Parser::with_config(input_data.as_bytes(), json_config);
            parser.parse_value()?;
        }
        zparse::Format::Toml => {
            let mut parser = zparse::toml::Parser::new(input_data.as_bytes());
            parser.parse()?;
        }
        zparse::Format::Yaml => {
            let mut parser = zparse::yaml::Parser::new(input_data.as_bytes());
            parser.parse()?;
        }
        zparse::Format::Xml => {
            let mut parser = zparse::xml::Parser::new(input_data.as_bytes());
            parser.parse()?;
        }
    }

    if args.print_output {
        write_output(&args.output, input_data.as_bytes())?;
    } else {
        write_output(&args.output, b"ok\n")?;
    }
    Ok(())
}

fn run_convert(args: ConvertArgs) -> Result<()> {
    let input_data = read_input(&args.input)?;
    let from = resolve_format(args.from, &args.input)?;
    let json_config = json_config_from_flags(args.json_comments, args.json_trailing_commas);
    let options = zparse::ConvertOptions { json: json_config };
    let output = zparse::convert_with_options(&input_data, from, args.to.into(), &options)?;

    if args.print_output {
        write_output(&args.output, output.as_bytes())?;
    } else {
        write_output(&args.output, b"ok\n")?;
    }
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

fn normalize_flag_input(input: Option<PathBuf>) -> Option<PathBuf> {
    match input {
        Some(value) if value.as_os_str() == "-" => None,
        other => other,
    }
}

fn resolve_format(from: Option<FormatArg>, input: &Option<PathBuf>) -> Result<zparse::Format> {
    from.map(zparse::Format::from)
        .or_else(|| input.as_ref().and_then(zparse::detect_format_from_path))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "could not infer input format; pass --from or provide an input file with extension"
            )
        })
}

fn json_config_from_flags(allow_comments: bool, allow_trailing_commas: bool) -> zparse::JsonConfig {
    zparse::JsonConfig::default()
        .with_comments(allow_comments)
        .with_trailing_commas(allow_trailing_commas)
}
