use wast::parser::{self, ParseBuffer};
use wast::Wat;

use crate::module::parse_module_struct_from_ast;
use crate::pretty_print::pretty_print_ast;

mod ast;
mod module;
mod pretty_print;

pub fn parse_wast_string(wast_string: &str, print_ast: bool) {
    let parse_buffer = ParseBuffer::new(wast_string).expect("Failed to lex");
    let wat = parser::parse::<Wat>(&parse_buffer).expect("Failed to parse");

    if print_ast {
        pretty_print_ast(&wat);
    }

    println!();

    let module = parse_module_struct_from_ast(&wat);

    println!("{:?}", module);
}
