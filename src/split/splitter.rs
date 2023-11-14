use std::io::Write;

use wast::core::{Func, FuncKind, Instruction, ModuleField, ModuleKind};
use wast::Wat;

use crate::split::instruction_analysis::{
    index_of_scope_end, BlockInstructionType, InstructionType, MemoryInstructionType, SplitType,
    StackEffect, StackValue,
};
use crate::split::splitter::ScopeType::Block;
use crate::split::utils::*;
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
                    handle_top_level_func(func, &expression.instrs, &mut transformer)?;
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
    instructions: &[Instruction],
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
    setup_func(&name, instructions, transformer);
    transformer.utx_function_names.push(name.clone());
    let mut deferred_splits = handle_instructions(
        &name,
        func_offset,
        instructions,
        0,
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

fn setup_func(name: &str, instructions: &[Instruction], transformer: &mut WatEmitter) {
    transformer.emit_utx_func_signature(name);
    //transformer.utx_function_names.push(name.into());
    transformer.emit_locals_if_neccessary(instructions);
}

fn handle_instructions<'a>(
    name: &str,
    func_offset: usize,
    instructions: &'a [Instruction<'a>],
    instruction_offset: usize,
    mut stack: Vec<StackValue>,
    mut scopes: Vec<Scope>,
    split_count: usize,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    let mut deferred_splits: Vec<DeferredSplit> = Vec::default();
    transformer.current_scope_level = scopes.len();
    for (i, instruction) in instructions.iter().enumerate() {
        let ty = InstructionType::from(instruction);
        if let Some((split_type, culprit_instruction)) =
            ty.needs_split(&stack, &scopes, transformer.skip_safe_splits)?
        {
            let local_offset = i + 1;
            if let Some(new_split) = handle_pre_split(
                name,
                func_offset,
                culprit_instruction,
                &instructions[local_offset..],
                instruction_offset + local_offset,
                split_count + deferred_splits.len(),
                &mut stack,
                &scopes,
                transformer,
            ) {
                deferred_splits.push(new_split);
            }
            match split_type {
                SplitType::Block => {
                    transformer.emit_instruction("return".into(), None);
                    let scope_end = index_of_scope_end(&instructions[i..])? + i;
                    let mut new_splits = handle_instructions(
                        name,
                        func_offset,
                        &instructions[scope_end..],
                        instruction_offset + scope_end,
                        stack,
                        scopes, // This might not be correct
                        split_count + deferred_splits.len(),
                        transformer,
                    )?;
                    deferred_splits.append(&mut new_splits);
                    return Ok(deferred_splits);
                }
                SplitType::Normal => {
                    break;
                }
                SplitType::Loop => return Err("Loop splitting is not yet supported"),
            }
        } else if let InstructionType::Benign(Some(ty)) = ty {
            let stack_start = stack.len();
            match ty {
                BlockInstructionType::Block(name) => {
                    // TODO - save stack (sigh)
                    scopes.push(Scope {
                        ty: Block,
                        name,
                        stack_start,
                    });
                    transformer
                        .emit_instruction_from_function(func_offset, instruction_offset + i)?;
                    transformer.current_scope_level += 1;
                    continue;
                }
                BlockInstructionType::End => {
                    let scope = scopes
                        .pop()
                        .ok_or("Unbalanced scopes - tried to remove top-level scope")?;
                    match scope.ty {
                        Block => {
                            // Slice off popped scope stack
                            stack = stack[..scope.stack_start].to_vec();
                        }
                    }
                    transformer.current_scope_level -= 1;
                }
                BlockInstructionType::Loop(_) => {
                    return Err("Loop instructions are not yet supported");
                }
            }
        }
        StackEffect::from(instruction).update_stack(&mut stack)?;
        transformer.emit_instruction_from_function(func_offset, instruction_offset + i)?;
    }
    transformer.emit_end_func();
    Ok(deferred_splits)
}

fn handle_defered_split<'a>(
    mut deferred_split: DeferredSplit<'a>,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    setup_func(
        &deferred_split.name,
        deferred_split.instructions,
        transformer,
    );
    if deferred_split.scopes.is_empty() {
        transformer.emit_restore_stack(&deferred_split.stack, 0, deferred_split.stack.len());
    } else {
        transformer.current_scope_level = 0;
        let mut curr_stack_base = 0;
        for scope in &deferred_split.scopes {
            match scope.ty {
                Block => {
                    transformer.emit_restore_stack(
                        &deferred_split.stack,
                        curr_stack_base,
                        scope.stack_start,
                    );
                    curr_stack_base = scope.stack_start;
                    let instruction = if let Some(name) = &scope.name {
                        format!("(block ${name}")
                    } else {
                        "(block".into()
                    };
                    transformer.emit_instruction(&instruction, None);
                    transformer.current_scope_level += 1;
                }
            }
        }
    }
    let post_split: Vec<(String, Option<String>)> = match deferred_split.culprit_instruction {
        MemoryInstructionType::Load { ty, .. } => {
            deferred_split.stack.push(StackValue { ty, is_safe: false });
            let load_data_type = format!("{}.load", ty.as_str());
            vec![
                ("local.get $utx".into(), Some("Restore load address".into())),
                ("i32.load".into(), None),
                (load_data_type, None),
            ]
        }
        MemoryInstructionType::Store { ty, .. } => {
            let store_data_type = format!("{}.store", ty.as_str());
            let load_data_type = format!("{}.load", ty.as_str());
            vec![
                (
                    "local.get $utx".into(),
                    Some("Restore store address".into()),
                ),
                ("i32.load".into(), None),
                (
                    "local.get $state".into(),
                    Some("Restore store value".into()),
                ),
                (load_data_type, None),
                (store_data_type, None),
            ]
        }
    };

    for (post_split_instr, annotation) in post_split {
        transformer.emit_instruction(&post_split_instr, annotation);
    }

    // This call recurses indirectly
    handle_instructions(
        &deferred_split.name,
        deferred_split.func_offset,
        deferred_split.instructions,
        deferred_split.instruction_offset,
        deferred_split.stack,
        deferred_split.scopes,
        0,
        transformer,
    )
}

fn handle_pre_split<'a>(
    base_name: &str,
    func_offset: usize,
    culprit_instruction: MemoryInstructionType,
    instructions: &'a [Instruction],
    instruction_offset: usize,
    split_count: usize,
    stack: &mut Vec<StackValue>,
    scopes: &[Scope],
    transformer: &mut WatEmitter,
) -> Option<DeferredSplit<'a>> {
    let pre_split = match culprit_instruction {
        MemoryInstructionType::Load { offset, .. } => {
            stack.pop();
            let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
            let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
            let offset_const = format!("i32.const {offset}");
            vec![
                (set_address, Some("Save address for load".into())),
                ("local.get $utx".into(), None),
                (get_address, None),
                (offset_const, Some("Convert =offset to value".into())),
                ("i32.add".into(), None),
                ("i32.store".into(), None),
            ]
        }
        MemoryInstructionType::Store { ty, offset } => {
            stack.pop();
            stack.pop();
            let ty = ty.as_str();
            let stack_juggler_local_name = format!("{ty}_{STACK_JUGGLER_NAME}");
            let set_value = format!("local.set ${stack_juggler_local_name}");
            let get_value = format!("local.get ${stack_juggler_local_name}");
            let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
            let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
            let store_data_type = format!("{ty}.store");
            let offset_const = format!("i32.const {offset}");
            vec![
                (set_value, Some("Save value for store".into())),
                (set_address, Some("Save address for store".into())),
                ("local.get $state".into(), None),
                (get_value, None),
                (store_data_type, None),
                ("local.get $utx".into(), None),
                (get_address, None),
                (offset_const, Some("Convert =offset to value".into())),
                ("i32.add".into(), None),
                ("i32.store".into(), None),
            ]
        }
    };

    for (pre_split_instr, annotation) in pre_split {
        transformer.emit_instruction(&pre_split_instr, annotation);
    }
    transformer.emit_instruction("local.get $utx".into(), Some("Save naddr = 1".into()));
    transformer.emit_instruction(&format!("i32.const 1"), None);
    transformer.emit_instruction("i32.store8 offset=63".into(), None);
    let stack_start = scopes.last().map(|scope| scope.stack_start).unwrap_or(0);

    transformer.emit_save_stack(&stack, stack_start);

    // TODO - identify by culprit instruction, not name
    let name = format!("{base_name}_{split_index}", split_index = split_count + 1);
    let existing_index = transformer
        .utx_function_names
        .iter()
        .position(|existing_name| existing_name.eq(&name));
    let index = existing_index.unwrap_or(transformer.utx_function_names.len()) + 1;
    transformer.emit_instruction(
        &format!("i32.const {index}"),
        Some("Return index to next microtransaction".into()),
    );

    match existing_index {
        None => {
            transformer.utx_function_names.push(name.clone());
            Some(DeferredSplit {
                name,
                func_offset,
                culprit_instruction,
                instructions,
                instruction_offset,
                stack: stack.to_vec(),
                scopes: scopes.to_vec(),
            })
        }
        Some(_) => None,
    }
}

#[derive(Clone)]
struct DeferredSplit<'a> {
    name: String,
    func_offset: usize,
    culprit_instruction: MemoryInstructionType,
    instructions: &'a [Instruction<'a>],
    instruction_offset: usize,
    stack: Vec<StackValue>,
    scopes: Vec<Scope>,
}

#[derive(Clone)]
pub struct Scope {
    pub ty: ScopeType,
    name: Option<String>,
    stack_start: usize,
}

#[derive(Clone)]
pub enum ScopeType {
    Block,
}
