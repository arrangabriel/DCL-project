use wast::{parser, Wat};

use wast::parser::ParseBuffer;

use ast_parsing::get_module_data_from_ast;
use ast_parsing::pretty_print_ast;

mod module_data;

mod ast_parsing;

pub fn parse_wast_string(wast_string: &str, print_ast: bool) -> Result<(), wast::Error> {
    let buffer = ParseBuffer::new(wast_string)?;
    let wat = parser::parse::<Wat>(&buffer)?;

    if print_ast {
        pretty_print_ast(&wat);
        println!();
    }

    let module = get_module_data_from_ast(&wat);

    println!("{:?}", module);
    Ok(())
}
