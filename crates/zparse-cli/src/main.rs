use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "zparse",
    version,
    about = "Parse and convert JSON/JSONC/CSV/TOML/YAML/XML",
    args_conflicts_with_subcommands = true,
    after_help = "Examples:\n  zparse --parse input.json --print-output\n  zparse --convert input.json --from json --to toml\n  zparse convert --from csv --to json input.csv\n  zparse parse --from json input.json\n  cat input.xml | zparse parse --from xml"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    /// Parse input and validate (top-level mode)
    #[arg(long, value_name = "INPUT", num_args = 0..=1, default_missing_value = "-", conflicts_with = "convert")]
    parse: Option<PathBuf>,
    /// Convert between formats (top-level mode)
    #[arg(long, value_name = "INPUT", num_args = 0..=1, default_missing_value = "-", conflicts_with = "parse")]
    convert: Option<PathBuf>,
    /// Input format (json, jsonc, csv, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    from: Option<FormatArg>,
    /// Output format (json, csv, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    to: Option<OutputFormatArg>,
    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
    /// Write input/converted output instead of "ok"
    #[arg(long = "print-output")]
    print_output: bool,
    /// Allow JSON comments (// and /* */)
    #[arg(long)]
    json_comments: bool,
    /// Allow trailing commas in JSON
    #[arg(long)]
    json_trailing_commas: bool,
    /// CSV field delimiter as a single character (default: ,)
    #[arg(long, value_name = "CHAR")]
    csv_delimiter: Option<char>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate an input file or stdin
    Parse(ParseArgs),
    /// Convert between formats
    Convert(ConvertArgs),
}

#[derive(Debug, Parser)]
struct ParseArgs {
    /// Input file (defaults to stdin)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    /// Input format (json, jsonc, csv, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    from: Option<FormatArg>,
    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
    /// Write input content instead of "ok"
    #[arg(long = "print-output")]
    print_output: bool,
    /// Allow JSON comments (// and /* */)
    #[arg(long)]
    json_comments: bool,
    /// Allow trailing commas in JSON
    #[arg(long)]
    json_trailing_commas: bool,
    /// CSV field delimiter as a single character (default: ,)
    #[arg(long, value_name = "CHAR")]
    csv_delimiter: Option<char>,
}

#[derive(Debug, Parser)]
struct ConvertArgs {
    /// Input file (defaults to stdin)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,
    /// Input format (json, jsonc, csv, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    from: Option<FormatArg>,
    /// Output format (json, csv, toml, yaml, xml)
    #[arg(short, long, value_enum)]
    to: OutputFormatArg,
    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
    /// Write converted output instead of "ok"
    #[arg(long = "print-output")]
    print_output: bool,
    /// Allow JSON comments (// and /* */)
    #[arg(long)]
    json_comments: bool,
    /// Allow trailing commas in JSON
    #[arg(long)]
    json_trailing_commas: bool,
    /// CSV field delimiter as a single character (default: ,)
    #[arg(long, value_name = "CHAR")]
    csv_delimiter: Option<char>,
}

#[derive(Clone, Debug, ValueEnum)]
enum FormatArg {
    Json,
    Jsonc,
    Csv,
    Toml,
    #[value(alias = "yml")]
    Yaml,
    Xml,
}

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormatArg {
    Json,
    Csv,
    Toml,
    #[value(alias = "yml")]
    Yaml,
    Xml,
}

impl From<FormatArg> for zparse::Format {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Json | FormatArg::Jsonc => zparse::Format::Json,
            FormatArg::Csv => zparse::Format::Csv,
            FormatArg::Toml => zparse::Format::Toml,
            FormatArg::Yaml => zparse::Format::Yaml,
            FormatArg::Xml => zparse::Format::Xml,
        }
    }
}

impl From<OutputFormatArg> for zparse::Format {
    fn from(value: OutputFormatArg) -> Self {
        match value {
            OutputFormatArg::Json => zparse::Format::Json,
            OutputFormatArg::Csv => zparse::Format::Csv,
            OutputFormatArg::Toml => zparse::Format::Toml,
            OutputFormatArg::Yaml => zparse::Format::Yaml,
            OutputFormatArg::Xml => zparse::Format::Xml,
        }
    }
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
            csv_delimiter: args.csv_delimiter,
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
            csv_delimiter: args.csv_delimiter,
        };
        return run_convert(convert_args);
    }

    bail!("no command specified; use a subcommand or --parse/--convert");
}

fn run_parse(args: ParseArgs) -> Result<()> {
    let input_data = read_input(&args.input)?;
    let (from, is_jsonc) = resolve_format(args.from, &args.input)?;
    let json_config =
        json_config_from_flags(is_jsonc, args.json_comments, args.json_trailing_commas);

    match from {
        zparse::Format::Json => {
            let mut parser = zparse::json::Parser::with_config(input_data.as_bytes(), json_config);
            parser.parse_value()?;
        }
        zparse::Format::Csv => {
            let config = csv_config_from_flags(args.csv_delimiter)?;
            let mut parser = zparse::csv::Parser::with_config(input_data.as_bytes(), config);
            parser.parse()?;
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
    let (from, is_jsonc) = resolve_format(args.from, &args.input)?;
    let json_config =
        json_config_from_flags(is_jsonc, args.json_comments, args.json_trailing_commas);
    let csv_config = csv_config_from_flags(args.csv_delimiter)?;
    let options = zparse::ConvertOptions {
        json: json_config,
        csv: csv_config,
    };
    let to = args.to.into();
    let output = zparse::convert_with_options(&input_data, from, to, &options)?;

    if args.print_output {
        write_output(&args.output, output.as_bytes())?;
    } else {
        if let Some(path) = &args.output {
            write_output(&Some(path.clone()), output.as_bytes())?;
        }

        let mut stdout = io::stdout();
        stdout
            .write_all(b"ok\n")
            .context("failed to write stdout")?;
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

fn resolve_format(
    from: Option<FormatArg>,
    input: &Option<PathBuf>,
) -> Result<(zparse::Format, bool)> {
    let format_arg = from
        .or_else(|| {
            input.as_ref().and_then(|path| {
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("jsonc"))
                    .unwrap_or(false)
                {
                    Some(FormatArg::Jsonc)
                } else {
                    zparse::detect_format_from_path(path).map(|fmt| match fmt {
                        zparse::Format::Json => FormatArg::Json,
                        zparse::Format::Csv => FormatArg::Csv,
                        zparse::Format::Toml => FormatArg::Toml,
                        zparse::Format::Yaml => FormatArg::Yaml,
                        zparse::Format::Xml => FormatArg::Xml,
                    })
                }
            })
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "could not infer input format; pass --from or provide an input file with extension"
            )
        })?;

    let is_jsonc = matches!(format_arg, FormatArg::Jsonc);
    let zparse_format = zparse::Format::from(format_arg);
    Ok((zparse_format, is_jsonc))
}

fn json_config_from_flags(
    is_jsonc: bool,
    allow_comments: bool,
    allow_trailing_commas: bool,
) -> zparse::JsonConfig {
    let mut config = zparse::JsonConfig::default()
        .with_comments(allow_comments)
        .with_trailing_commas(allow_trailing_commas);

    if is_jsonc {
        config = config.with_comments(true).with_trailing_commas(true);
    }

    config
}

fn csv_config_from_flags(delimiter: Option<char>) -> Result<zparse::CsvConfig> {
    match delimiter {
        None => Ok(zparse::CsvConfig::default()),
        Some(ch) => {
            if !ch.is_ascii() {
                bail!("CSV delimiter must be an ASCII character");
            }
            let byte = ch as u8;
            if matches!(byte, b'\n' | b'\r' | b'"') {
                bail!(
                    "CSV delimiter {:?} conflicts with record separators or quoting rules",
                    ch
                );
            }
            Ok(zparse::CsvConfig::default().with_delimiter(byte))
        }
    }
}
