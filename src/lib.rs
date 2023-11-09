use std::io;

use wast::parser::ParseBuffer;
use wast::{parser, Wat};

use crate::ast_parsing::emit_transformed_wat;

mod ast_parsing;

pub fn parse_wast_string(
    wast_string: &str,
    print_ast: bool,
    skip_safe_splits: bool,
) -> Result<(), String> {
    let buffer = ParseBuffer::new(wast_string).map_err(|err| err.message())?;
    let wat = parser::parse::<Wat>(&buffer).map_err(|err| err.message())?;

    if print_ast {
        println!("{}", wast_string);
    }

    let writer = io::stdout();
    emit_transformed_wat(&wat, wast_string, Box::new(writer), skip_safe_splits)?;
    Ok(())
}
