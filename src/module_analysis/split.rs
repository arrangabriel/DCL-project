use wast::core::Instruction;

use crate::module_analysis::marking::mark_functions;
use crate::module_data::Function;

pub fn split_unsafe_functions<'a>(functions: &'a [Function]) -> Vec<Function<'a>> {
    let (safe_functions, unsafe_functions) = mark_functions(functions);

    let mut split_functions: Vec<Function> = Vec::new();
    for (func, _) in safe_functions {
        split_functions.push(func.shallow_clone());
    }

    let mut func_no = 0;
    for (func, marked_instructions) in unsafe_functions {
        let mut instruction_splits: Vec<&[Instruction]> = Vec::new();
        let mut prev_split: usize = 0;
        for (i, &(_, is_marked)) in marked_instructions.iter().enumerate() {
            if is_marked {
                if !(prev_split..i).is_empty() {
                    instruction_splits.push(&func.instructions[prev_split..i]);
                }
                prev_split = i;
            }
        }
        instruction_splits.push(&func.instructions[prev_split..]);

        let base_id = func.id.clone().unwrap_or_else(|| {
            func_no += 1;
            format!("function_{}", func_no)
        });
        for (i, &instruction_split) in instruction_splits.iter().enumerate() {
            // TODO - if this is not the last function in the split-chain we need to insert the next call
            let id = if i == 0 {
                Some(base_id.clone())
            } else {
                Some(format!("{}_{}", base_id.as_str(), i))
            };

            let new_func = Function {
                id,
                signature: func.signature.clone(), // This isn't correct, TODO
                instructions: &instruction_split,
            };

            split_functions.push(new_func);
        }
    }

    split_functions
}
