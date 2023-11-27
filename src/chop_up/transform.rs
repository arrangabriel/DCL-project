use std::io::Write;

use wast::core::{Func, ModuleField, ModuleKind};
use wast::Wat;

use crate::chop_up::function;
use crate::chop_up::function::Function;
use crate::chop_up::instruction::{
    BenignInstructionType, BlockInstructionType, DataType, InstructionType,
};
use crate::chop_up::split::{DeferredSplit, handle_deferred_split, setup_split};
use crate::chop_up::constants::MODULE_MEMBER_INDENT;
use crate::chop_up::emit::WatEmitter;
use crate::chop_up::instruction_stream::Instruction;

pub fn emit_transformed_wat(
    wat: &Wat,
    lines: &[&str],
    writer: &mut dyn Write,
    skip_safe_splits: bool,
    state_size: usize,
    explain_splits: bool,
) -> Result<(), &'static str> {
    let module_fields = match wat {
        Wat::Module(module) => match &module.kind {
            ModuleKind::Text(fields) => Ok(fields),
            ModuleKind::Binary(_) => Err("ModuleKind is binary"),
        },
        Wat::Component(_) => Err("Input module is component"),
    }?;

    let mut transformer = WatEmitter::new(writer, state_size, skip_safe_splits, explain_splits);
    transformer.emit_module();

    let mut functions = Vec::default();
    let mut saved_offsets = Vec::default();
    for field in module_fields {
        match field {
            ModuleField::Func(func) => functions.push(extract_function(func, lines)?),
            ModuleField::Export(export) => saved_offsets.push(export.span.offset()),
            ModuleField::Type(ty) => saved_offsets.push(ty.span.offset()),
            ModuleField::Global(global) => saved_offsets.push(global.span.offset()),
            _ => { /* Other module fields might need to be handled at a later date */ }
        }
    }

    let mut deferred_splits = Vec::default();
    for func in &functions {
        let mut new_splits = handle_top_level_func(func, &mut transformer)?;
        deferred_splits.append(&mut new_splits);
    }

    while !deferred_splits.is_empty() {
        deferred_splits = deferred_splits
            .drain(..)
            .flat_map(|split| handle_deferred_split(split, &mut transformer).unwrap())
            .collect();
    }

    for saved_offset in saved_offsets {
        let line = function::get_line_from_offset(lines, saved_offset);
        let extra_parens = line.chars().fold(0, |v, c| {
            v + match c {
                '(' => -1,
                ')' => 1,
                _ => 0,
            }
        }) as usize;
        transformer.writeln(
            line[..line.len() - extra_parens].trim(),
            MODULE_MEMBER_INDENT,
        );
    }

    transformer.emit_end_module();
    Ok(())
}

fn extract_function<'a>(func: &'a Func, lines: &'a [&str]) -> Result<Function<'a>, &'static str> {
    Function::new(func, lines)
}

fn handle_top_level_func<'a>(
    func: &'a Function,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    if func.ignore() {
        transformer.emit_function(func);
        return Ok(Vec::default());
    }
    setup_func(
        &func.name,
        &func.instructions,
        &func.local_types,
        transformer,
    );
    transformer.utx_function_names.push((0, func.name.clone()));
    handle_instructions(
        &func.name,
        &func.instructions,
        &func.local_types,
        0,
        transformer,
    )
}

pub fn setup_func(
    name: &str,
    instructions: &[Instruction],
    locals: &[DataType],
    transformer: &mut WatEmitter,
) {
    transformer.emit_utx_func_signature(name);
    transformer.emit_locals(instructions, locals);
}

pub fn handle_instructions<'a>(
    name: &str,
    instructions: &'a [Instruction],
    locals: &[DataType],
    split_count: usize,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    let deferred_splits: Vec<DeferredSplit> = Vec::default();
    for (i, instruction) in instructions.iter().enumerate() {
        transformer.current_scope_level = instruction.scopes.len();
        let ty = InstructionType::from(instruction);
        match ty {
            InstructionType::Memory(ty) => {
                if ty.needs_split(
                    &instruction.stack,
                    transformer.skip_safe_splits,
                )? {
                    return setup_split(
                        name,
                        split_count + deferred_splits.len(),
                        &instructions[i + 1..],
                        locals,
                        (instruction, ty, instruction.index),
                        transformer,
                    );
                }
            }
            InstructionType::Benign(ty) => {
                match ty {
                    BenignInstructionType::Block(ty) => match ty {
                        BlockInstructionType::Block(_) => {
                            // Handle the special case of blocks being emitted on the previous indent
                            transformer.current_scope_level -= 1;
                            let prev_stack_start = instruction
                                .scopes
                                .len()
                                .checked_sub(2)
                                .map(|i| instruction.scopes[i].stack_start)
                                .unwrap_or(0);
                            transformer.emit_save_stack_and_locals(
                                transformer.stack_base,
                                &instruction.stack,
                                prev_stack_start,
                                true,
                                locals,
                            );
                        }
                        BlockInstructionType::End => {}
                    },
                    BenignInstructionType::IndexedLocal(ty, index) => {
                        // After changing function signatures:
                        // tx, state -> tx, utx, state
                        // all locals after the first have to be incremented by one
                        let instruction_str =
                            format!("local.{ty_str} {index}", ty_str = ty.as_str());
                        transformer.emit_instruction(&instruction_str, None);
                        continue;
                    }
                    BenignInstructionType::Return => {
                        if instruction.stack.is_empty() {
                            transformer.emit_instruction("i32.const 0", Some("Return NULL".into()));
                        }
                    }
                    BenignInstructionType::Other => {}
                }
            }
        }
        transformer.emit_instruction(instruction.raw_text, None);
    }
    transformer.emit_end_func();
    Ok(deferred_splits)
}
