use std::io::Write;

use anyhow::Result;
use wast::parser::{parse, ParseBuffer};

use crate::chop_up::emit_transformed_wat;

mod chop_up;

pub fn transform_wat_string(
    wast_string: &str,
    writer: &mut dyn Write,
    state_size: usize,
    skip_safe_splits: bool,
    explain_splits: bool,
) -> Result<()> {
    let buffer = ParseBuffer::new(wast_string)?;
    let wat = parse(&buffer)?;
    emit_transformed_wat(
        &wat,
        &wast_string.split('\n').collect::<Vec<&str>>(),
        writer,
        skip_safe_splits,
        state_size,
        explain_splits,
    )
}
