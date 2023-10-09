use crate::module_analysis::marking::{mark_functions, MarkedFunction};
use crate::module_data::Function;

pub fn print_functions(functions: &[Function], indentation: usize) {
    let prefix = "\t".repeat(indentation);
    for func in functions {
        println!("{prefix}{:?}", get_func_name(func, "anonymous"));
        for instruction in func.instructions {
            println!("{prefix}\t{:?}", instruction);
        }
    }
}

pub fn print_with_safety(functions: &[Function]) {
    let (safe_functions, unsafe_functions) = mark_functions(functions);

    println!("Safe functions:");
    print_marked_functions(safe_functions);

    println!("Unsafe functions:");
    print_marked_functions(unsafe_functions);
}

fn print_marked_functions(functions: Vec<MarkedFunction>) {
    for (func, instructions) in functions {
        println!("\t{:?}", get_func_name(func, "anonymous"));
        for (instruction, is_unsafe) in instructions {
            let postfix = if is_unsafe { "   <- UNSAFE" } else { "" };
            println!("\t\t{:?}{}", instruction, postfix);
        }
        println!();
    }
}

pub fn get_func_name<'a>(func: &'a Function<'a>, default: &'a str) -> &'a str {
    func.id.as_ref().map(|id| id.as_str()).unwrap_or(default)
}
