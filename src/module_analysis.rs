use wast::core::Instruction::{self, *};

use crate::module_data::Function;

type MarkedInstruction<'a> = (&'a Instruction<'a>, bool);

pub fn print_accessors(functions: &[Function]) {
    let mut safe_functions = Vec::<(&str, Vec<MarkedInstruction>)>::new();
    let mut unsafe_functions = Vec::<(&str, Vec<MarkedInstruction>)>::new();

    for func in functions {
        let instructions_marked: Vec<MarkedInstruction> = func
            .instructions
            .iter()
            .map(|instruction| (instruction, is_mem_access_instruction(instruction)))
            .collect();
        let func_name = func
            .id
            .as_ref()
            .map(|id| id.as_str())
            .unwrap_or("anonymous");

        let func_is_unsafe = instructions_marked
            .iter()
            .any(|(_, mem_instruction)| *mem_instruction);

        if func_is_unsafe {
            &mut unsafe_functions
        } else {
            &mut safe_functions
        }
        .push((func_name, instructions_marked));
    }

    println!("Safe functions:");
    for (func_name, instructions) in safe_functions {
        println!("\t{:?}", func_name);
        for (instruction, _) in instructions {
            println!("\t\t{:?}", instruction)
        }
        println!();
    }

    println!("Unsafe functions:");
    for (func_name, instructions) in unsafe_functions {
        println!("\t{:?}", func_name);
        for (instruction, is_unsafe) in instructions {
            if is_unsafe {
                println!("\t\t{:?}   <- UNSAFE", instruction)
            } else {
                println!("\t\t{:?}", instruction)
            }
        }
        println!();
    }
}

fn is_mem_access_instruction(instruction: &Instruction) -> bool {
    match instruction {
        GlobalGet(_) | GlobalSet(_) | TableGet(_) | TableSet(_) | I32Load(_) | I64Load(_)
        | F32Load(_) | F64Load(_) | I32Load8s(_) | I32Load8u(_) | I32Load16s(_) | I32Load16u(_)
        | I64Load8s(_) | I64Load8u(_) | I64Load16s(_) | I64Load16u(_) | I64Load32s(_)
        | I64Load32u(_) | I32Store(_) | I64Store(_) | F32Store(_) | F64Store(_) | I32Store8(_)
        | I32Store16(_) | I64Store8(_) | I64Store16(_) | I64Store32(_) | MemorySize(_)
        | MemoryGrow(_) | MemoryInit(_) | MemoryCopy(_) | MemoryFill(_) | MemoryDiscard(_)
        | DataDrop(_) | ElemDrop(_) | TableInit(_) | TableCopy(_) | TableFill(_) | TableSize(_)
        | TableGrow(_) => true,
        _ => false,
    }
}
