use std::io;

use wast::parser::ParseBuffer;
use wast::{parser, Wat};

use ast_parsing::transform_emit_ast;

mod ast_parsing;

pub fn parse_wast_string(wast_string: &str, print_ast: bool) -> Result<(), wast::Error> {
    let buffer = ParseBuffer::new(wast_string)?;
    let wat = parser::parse::<Wat>(&buffer)?;

    if print_ast {
        println!("{}", wast_string);
    }

    let writer = io::stdout();
    transform_emit_ast(&wat, wast_string, Box::new(writer));

    Ok(())
}
