use std::io::Write;

use wast::core::{Func, FuncKind, Instruction, ModuleField, ModuleKind};
use wast::Wat;

use crate::split::instruction_analysis::{
    BenignInstructionType, BlockInstructionType, DataType, InstructionType, StackEffect, StackValue,
};
use crate::split::split::{handle_defered_split, setup_split, DeferredSplit};
use crate::split::utils::{gen_random_func_name, IGNORE_FUNC_PREFIX};
use crate::split::wat_emitter::WatEmitter;

pub fn emit_transformed_wat(
    wat: &Wat,
    raw_text: &str,
    writer: Box<dyn Write>,
    skip_safe_splits: bool,
) -> Result<(), &'static str> {
    let module_fields = match wat {
        Wat::Module(module) => match &module.kind {
            ModuleKind::Text(fields) => Ok(fields),
            ModuleKind::Binary(_) => Err("ModuleKind is binary"),
        },
        Wat::Component(_) => Err("Input module is component"),
    }?;

    let mut transformer = WatEmitter::new(raw_text, writer, skip_safe_splits);
    transformer.emit_module();

    let mut functions_with_instructions = Vec::default();
    let mut exports = Vec::default();
    let mut types = Vec::default();
    let mut globals = Vec::default();
    for field in module_fields {
        match field {
            ModuleField::Func(func) => {
                if let FuncKind::Inline {
                    expression,
                    locals: _,
                } = &func.kind
                {
                    // Maybe this should happen inside of WatEmitter?
                    let instruction_base_index = transformer.get_function_instructions_index(func);
                    let instructions_with_index: Vec<(&Instruction, usize)> = expression
                        .instrs
                        .iter()
                        .enumerate()
                        .map(|(i, instruction)| (instruction, instruction_base_index + i))
                        .collect();
                    functions_with_instructions.push((func, instructions_with_index));
                }
            }
            ModuleField::Export(export) => {
                exports.push(export.span.offset());
            }
            ModuleField::Type(ty) => {
                types.push(ty.span.offset());
            }
            ModuleField::Global(global) => globals.push(global.span.offset()),
            _ => { /* Other module fields might need to be handled at a later date */ }
        }
    }

    let mut deferred_splits = Vec::default();
    for (func, instructions) in &functions_with_instructions {
        let mut new_splits = handle_top_level_func(func, instructions, &mut transformer)?;
        deferred_splits.append(&mut new_splits);
    }

    while !deferred_splits.is_empty() {
        deferred_splits = deferred_splits
            .drain(..deferred_splits.len())
            .flat_map(|split| handle_defered_split(split, &mut transformer).unwrap())
            .collect();
    }

    for global_offset in globals {
        transformer.emit_section(global_offset)?;
    }
    for export_offset in exports {
        transformer.emit_section(export_offset)?;
    }
    for type_offset in types {
        transformer.emit_section(type_offset)?;
    }

    transformer.emit_end_module();
    Ok(())
}

static UTX_NULL_RETURN: [&str; 1] = ["i32.const 0"];

fn handle_top_level_func<'a>(
    func: &Func,
    instructions_with_index: &'a [(&Instruction, usize)],
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    let name = match func.id.map(|id| id.name()) {
        None => gen_random_func_name(),
        Some(func_name) => {
            // Maybe we also need to skip functions that dont have the type (tx, state) -> void
            // but they also need to be split..
            // don't wupport them for now ;)
            if func_name.starts_with(IGNORE_FUNC_PREFIX) {
                transformer.emit_section(func.span.offset())?;
                return Ok(Vec::default());
            }
            func_name.into()
        }
    };
    let local_types = if let FuncKind::Inline { locals, .. } = &func.kind {
        locals
            .iter()
            .map(|local| DataType::from(local.ty))
            .collect()
    } else {
        Vec::default()
    };
    setup_func(&name, instructions_with_index, &local_types, transformer);
    transformer.utx_function_names.push((0, name.clone()));
    handle_instructions(
        &name,
        instructions_with_index,
        &local_types,
        Vec::default(),
        Vec::default(),
        0,
        &UTX_NULL_RETURN,
        transformer,
    )
}

pub fn setup_func(
    name: &str,
    instructions_with_index: &[(&Instruction, usize)],
    local_types: &[DataType],
    transformer: &mut WatEmitter,
) {
    transformer.emit_utx_func_signature(name);
    transformer.emit_locals(instructions_with_index, local_types);
}

pub fn handle_instructions<'a>(
    name: &str,
    instructions: &'a [(&Instruction<'a>, usize)],
    local_types: &[DataType],
    mut stack: Vec<StackValue>,
    mut scopes: Vec<Scope>,
    split_count: usize,
    tail_instructions: &'static [&'static str],
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    let deferred_splits: Vec<DeferredSplit> = Vec::default();
    transformer.current_scope_level = scopes.len();
    for (i, &(instruction, instruction_index)) in instructions.iter().enumerate() {
        let ty = InstructionType::from(instruction);
        match ty {
            InstructionType::Memory(ty) => {
                if let Some(split_type) =
                    ty.needs_split(&stack, &scopes, transformer.skip_safe_splits)?
                {
                    return setup_split(
                        name,
                        split_count + deferred_splits.len(),
                        // Only pass on instructions after the culprit
                        &instructions[i + 1..],
                        local_types,
                        (ty, instruction_index),
                        split_type,
                        stack,
                        &scopes,
                        deferred_splits,
                        tail_instructions,
                        transformer,
                    );
                }
            }
            InstructionType::Benign(ty) => {
                match ty {
                    BenignInstructionType::Block(ty) => {
                        let stack_start = stack.len();
                        match ty {
                            BlockInstructionType::Block(name) => {
                                let prev_stack_start =
                                    scopes.last().map(|scope| scope.stack_start).unwrap_or(0);
                                transformer.emit_save_stack(&stack, prev_stack_start, true);
                                scopes.push(Scope {
                                    ty: ScopeType::Block,
                                    name,
                                    stack_start,
                                });
                                transformer.emit_instruction_by_index(instruction_index)?;
                                transformer.current_scope_level += 1;
                                continue;
                            }
                            BlockInstructionType::End => {
                                let scope = scopes
                                    .pop()
                                    .ok_or("Unbalanced scopes - tried to remove top-level scope")?;
                                match scope.ty {
                                    ScopeType::Block => {
                                        // Slice off popped scope stack
                                        stack = stack[..scope.stack_start].to_vec();
                                    }
                                }
                                transformer.current_scope_level -= 1;
                            }
                        }
                    }
                    BenignInstructionType::IndexedLocal(ty, mut index) => {
                        // After changins function signatures:
                        // tx, state -> tx, utx, state
                        // all locals after the first have to be incremented by one
                        index += if index == 0 { 0 } else { 1 };
                        let instruction_str =
                            format!("local.{ty_str} {index}", ty_str = ty.as_str());
                        transformer.emit_instruction(&instruction_str, None);
                        StackEffect::from_instruction(instruction, local_types)
                            .update_stack(&mut stack)?;
                        continue;
                    }
                    BenignInstructionType::Return => {
                        if stack.is_empty() {
                            transformer.emit_instruction("i32.const 0", Some("Return NULL".into()));
                        }
                    }
                    BenignInstructionType::Other => {}
                }
            }
        }
        StackEffect::from_instruction(instruction, local_types).update_stack(&mut stack)?;
        transformer.emit_instruction_by_index(instruction_index)?;
    }
    for tail_instruction in tail_instructions {
        transformer.emit_instruction(tail_instruction, None);
    }
    transformer.emit_end_func();
    Ok(deferred_splits)
}

#[derive(Clone)]
pub struct Scope {
    pub ty: ScopeType,
    pub(crate) name: Option<String>,
    pub stack_start: usize,
}

#[derive(Clone)]
pub enum ScopeType {
    Block,
}
