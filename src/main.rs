use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{env, io};

use chop_up::transform_wat_string;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let program_name = args
        .get(0)
        .and_then(|name| name.split('/').last())
        .expect("Program name should always be an argument");

    let config = parse_config(&args[1..]).map_err(|err| {
        eprintln!(
            "\
Usage {program_name} [input_file] [state_size] [opts...]
Possible opts are:
  --skip-safe        optimize splits by skipping accesses to function arguments
  --explain-splits   add explanatory comments to transformed code
        "
        );
        anyhow!(err)
    })?;

    let file_path = Path::new(config.file_path);
    let mut wat_string = String::new();

    if !file_path.is_file() {
        return Err(anyhow!("No such file: {}", config.file_path));
    }

    File::open(file_path)
        .and_then(|mut file| file.read_to_string(&mut wat_string))
        .map_err(|err| anyhow!("Failed to read file: {err:?}"))?;

    transform_wat_string(
        wat_string.as_str(),
        &mut io::stdout(),
        config.state_size,
        config.skip_safe,
        config.explain_splits,
    )
}

struct Config<'a> {
    file_path: &'a str,
    state_size: usize,
    skip_safe: bool,
    explain_splits: bool,
}

impl<'a> Config<'a> {
    fn default(file_path: &'a str, state_size: usize) -> Self {
        Self {
            file_path,
            state_size,
            skip_safe: false,
            explain_splits: false,
        }
    }
}

fn parse_config(args: &[String]) -> Result<Config, String> {
    let file_path = args.get(0).ok_or("Missing file path")?;
    let state_size = args
        .get(1)
        .ok_or("Missing state size")?
        .parse()
        .map_err(|_| "State size must be a positive integer")?;
    let mut config = Config::default(file_path, state_size);

    for flag in args[2..].iter() {
        match flag.as_str() {
            "--skip-safe" => config.skip_safe = true,
            "--explain-splits" => config.explain_splits = true,
            _ => {
                return Err(format!("Unknown flag {flag}"));
            }
        }
    }

    Ok(config)
}
