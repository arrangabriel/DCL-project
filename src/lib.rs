use std::io;

use crate::split::emit_transformed_wat;
use wast::parser::ParseBuffer;
use wast::{parser, Wat};

mod split;

pub fn parse_wast_string(
    wast_string: &str,
    state_size: usize,
    skip_safe_splits: bool,
) -> Result<(), String> {
    let buffer = ParseBuffer::new(wast_string).map_err(|err| err.message())?;
    let wat = parser::parse::<Wat>(&buffer).map_err(|err| err.message())?;

    let writer = io::stdout();
    emit_transformed_wat(
        &wat,
        &wast_string.split("\n").collect::<Vec<&str>>(),
        Box::new(writer),
        skip_safe_splits,
        state_size,
    )?;
    Ok(())
}
