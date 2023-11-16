use std::io::Write;

use itertools::Itertools;
use wast::core::{Func, FuncKind, Instruction, ModuleField, ModuleKind, ValType};
use wast::Wat;

use crate::split::instruction_analysis::{
    BlockInstructionType, DataType, InstructionType, StackEffect, StackValue,
};
use crate::split::split::{handle_defered_split, setup_split, DeferredSplit};
use crate::split::utils::{gen_random_func_name, IGNORE_FUNC_PREFIX, MODULE_MEMBER_INDENT};
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
    for field in module_fields {
        match field {
            ModuleField::Func(func) => {
                if let FuncKind::Inline {
                    expression,
                    locals: _,
                } = &func.kind
                {
                    let instruction_base_index = transformer.get_function_instructions_index(func);
                    let instructions_with_index: Vec<(&Instruction, usize)> = expression
                        .instrs
                        .iter()
                        .enumerate()
                        .map(|(i, instruction)| (instruction, instruction_base_index + i))
                        .collect();
                    handle_top_level_func(func, &instructions_with_index, &mut transformer)?;
                }
            }
            ModuleField::Export(export) => {
                transformer.emit_section(export.span.offset(), MODULE_MEMBER_INDENT)?;
            }
            _ => { /* Other module fields might need to be handled at a later date */ }
        }
    }
    transformer.emit_end_module();
    Ok(())
}

fn handle_top_level_func(
    func: &Func,
    instructions_with_index: &[(&Instruction, usize)],
    transformer: &mut WatEmitter,
) -> Result<(), &'static str> {
    let name = match func.id.map(|id| id.name()) {
        None => gen_random_func_name(),
        Some(func_name) => {
            if func_name.starts_with(IGNORE_FUNC_PREFIX) {
                transformer.emit_section(func.span.offset(), MODULE_MEMBER_INDENT)?;
                return Ok(());
            }
            func_name.into()
        }
    };
    let func_offset = func.span.offset();
    setup_func(&name, instructions_with_index, transformer);

    transformer
        .utx_function_names
        .push((func_offset, name.clone())); // Here we should use some sort of default value

    if let FuncKind::Inline { locals, .. } = &func.kind {
        let local_types = locals
            .iter()
            .map(|local| {
                match local.ty {
                    ValType::I32 => DataType::I32,
                    ValType::I64 => DataType::I64,
                    ValType::F32 => DataType::F32,
                    ValType::F64 => DataType::F64,
                    _ => panic!("Unsupported local type {:?}", local.ty),
                }
                .as_str()
                .to_string() // Lame
            })
            .join(" ");
        if !local_types.is_empty() {
            transformer.emit_instruction(&format!("(local {local_types})"), None)
        }
    };
    // Maybe we return these, and handle them in bulk after emitting all top-level
    let mut deferred_splits = handle_instructions(
        &name,
        instructions_with_index,
        Vec::default(),
        Vec::default(),
        0,
        transformer,
    )?;

    while !deferred_splits.is_empty() {
        deferred_splits = deferred_splits
            .drain(..deferred_splits.len())
            .flat_map(|split| handle_defered_split(split, transformer).unwrap())
            .collect();
    }

    Ok(())
}

pub fn setup_func(
    name: &str,
    instructions_with_index: &[(&Instruction, usize)],
    transformer: &mut WatEmitter,
) {
    transformer.emit_utx_func_signature(name);
    transformer.emit_locals_if_neccessary(instructions_with_index);
}

pub fn handle_instructions<'a>(
    name: &str,
    instructions: &'a [(&Instruction<'a>, usize)],
    mut stack: Vec<StackValue>,
    mut scopes: Vec<Scope>,
    split_count: usize,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    let deferred_splits: Vec<DeferredSplit> = Vec::default();
    transformer.current_scope_level = scopes.len();
    for (i, &(instruction, instruction_index)) in instructions.iter().enumerate() {
        let ty = InstructionType::from(instruction);
        if let Some((split_type, culprit_instruction_type)) =
            ty.needs_split(&stack, &scopes, transformer.skip_safe_splits)?
        {
            return setup_split(
                name,
                split_count + deferred_splits.len(),
                // Only pass on instructions after the culprit
                &instructions[i + 1..],
                (culprit_instruction_type, instruction_index),
                split_type,
                stack,
                &scopes,
                deferred_splits,
                transformer,
            );
        } else if let InstructionType::Benign(Some(ty)) = ty {
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
        StackEffect::from(instruction).update_stack(&mut stack)?;
        transformer.emit_instruction_by_index(instruction_index)?;
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
