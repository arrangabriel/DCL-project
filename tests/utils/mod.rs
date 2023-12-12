use pretty_assertions::assert_eq;

use chop_up::transform_wat_string;

pub fn test_transform(input: &str, expected_output: &str) {
    let mut output_vec: Vec<u8> = Vec::new();
    transform_wat_string(input, &mut output_vec, 6, false, false).unwrap();
    let output_wat = String::from_utf8(output_vec).unwrap();
    assert_eq!(output_wat.trim(), expected_output.trim());
}
