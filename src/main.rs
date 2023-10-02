use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

use wasm_manipulation::parse_wast_string;

fn main() {
    let args: Vec<String> = env::args().collect();
    match parse_config(args) {
        None => {
            println!("Usage: [wasm-manipulator] file-path");
            exit(1)
        }
        Some(config) => {
            let file_path = Path::new(config.file_path.as_str());
            let mut file = File::open(file_path).expect("Failed to open file");
            let mut wat_string = String::new();
            file.read_to_string(&mut wat_string)
                .expect("Failed to read file");

            parse_wast_string(wat_string.as_str(), config.print);
        }
    }
}

struct Config {
    file_path: String,
    print: bool,
}

fn parse_config(mut args: Vec<String>) -> Option<Config> {
    args.remove(0);
    let file_path = args
        .iter()
        .position(|arg| !arg.starts_with("-"))
        .map(|pos| args.remove(pos))?;

    let print = check_flag(&mut args, "-p");

    Some(Config { file_path, print })
}

fn check_flag(args: &mut Vec<String>, flag: &str) -> bool {
    args.iter()
        .position(|arg| arg.as_str().eq(flag))
        .map(|pos| args.swap_remove(pos))
        .is_some()
}
