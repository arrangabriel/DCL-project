use std::{env, io};

use anyhow::{anyhow, Result};

use chop_up::{OutputFormat, run_analysis, run_split};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let program_name = args
        .get(0)
        .and_then(|name| name.split('/').last())
        .expect("Program name should always be an argument");

    let config = parse_config(&args[1..]).map_err(|err| {
        eprintln!("\
Usage {program_name} [command] [input_file] [opts...]");
        anyhow!(err)
    })?;

    match config {
        Config::ChopConfig { file_path, state_size, skip_safe, explain } => run_split(file_path, state_size, skip_safe, explain, &mut io::stdout()),
        Config::AnalyticsConfig { file_path, output_format } => run_analysis(file_path, output_format)
    }
}

enum Config<'a> {
    ChopConfig {
        file_path: &'a str,
        state_size: usize,
        skip_safe: bool,
        explain: bool,
    },
    AnalyticsConfig {
        file_path: &'a str,
        output_format: OutputFormat,
    },
}

fn parse_config(args: &[String]) -> Result<Config> {
    let subcommand = args.get(0).ok_or(anyhow!("Missing subcommand"))?;
    let file_path = args.get(1).ok_or(anyhow!("Missing file path"))?;
    match subcommand.as_str() {
        "split" => parse_split_config(file_path, &args[2..]),
        "analyze" => parse_analytics_config(file_path, &args[2..]),
        _ => Err(anyhow!("\
Unknown command {subcommand}
Possible commands are:
  split    split transactional code
  analyze  calculate analytics for wasm code")),
    }
}

fn parse_split_config<'a>(file_path: &'a str, args: &[String]) -> Result<Config<'a>> {
    let state_size = args
        .get(0)
        .ok_or(anyhow!("Missing state size"))?
        .parse()
        .map_err(|_| anyhow!("State size must be a positive integer"))?;
    let mut skip_safe = false;
    let mut explain = false;

    for flag in args[1..].iter() {
        match flag.as_str() {
            "--skip-safe" => skip_safe = true,
            "--explain" => explain = true,
            _ => {
                return Err(anyhow!("\
Unknown opt {flag}
Possible opts are:
  --skip-safe  optimize splits by skipping accesses to function arguments
  --explain    add explanatory comments to transformed code")
                );
            }
        }
    }

    Ok(Config::ChopConfig {
        file_path,
        state_size,
        skip_safe,
        explain,
    })
}

fn parse_analytics_config<'a>(file_path: &'a str, args: &[String]) -> Result<Config<'a>> {
    let output_format = match args.get(0)
        .ok_or(anyhow!("Missing output format"))?
        .as_str() {
        "standard" => OutputFormat::Normal,
        "csv" => OutputFormat::CSV,
        unknown_format => return Err(anyhow!("\
Unknown output format {unknown_format}
Supported formats:
  standard
  csv"))
    };

    if let Some(flag) = args[1..].first() {
        return Err(anyhow!("Unknown opt {flag}"));
    }

    Ok(Config::AnalyticsConfig {
        file_path,
        output_format,
    })
}
