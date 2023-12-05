use crate::chop_up::instruction::DataType;

pub const UTX_LOCALS: [DataType; 3] = [DataType::I32, DataType::I32, DataType::I32];
pub const ADDRESS_LOCAL_NAME: &str = "memory_address";
pub const STACK_JUGGLER_NAME: &str = "local";
pub const MODULE_MEMBER_INDENT: usize = 1;

pub fn count_parens(string: &str) -> i32 {
    string.chars().fold(0, |v, c| {
        v + match c {
            '(' => -1,
            ')' => 1,
            _ => 0,
        }
    })
}

pub fn get_line_index_from_offset<'a>(lines: &'a [&'a str], offset: usize) -> usize {
    let total_len = lines.iter().map(|l| l.len() + 1).sum();
    assert!(offset < total_len, "Offset provided was out of bounds");
    let mut line_end = 0;
    for (i, line) in lines.iter().enumerate() {
        line_end += line.len() + 1;
        if offset < line_end {
            return i;
        }
    }
    unreachable!()
}

pub fn get_line_from_offset<'a>(lines: &'a [&'a str], offset: usize) -> &'a str {
    lines[get_line_index_from_offset(lines, offset)]
}
