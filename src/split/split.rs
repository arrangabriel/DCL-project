use crate::split::function_analysis::{index_of_scope_end, SplitType, StackValue};
use crate::split::instruction_types::{DataType, Instruction, MemoryInstructionType};
use crate::split::transform::{handle_instructions, setup_func, Scope, ScopeType};
use crate::split::utils::*;
use crate::split::wat_emitter::WatEmitter;

pub fn setup_split<'a>(
    base_name: &str,
    base_split_count: usize,
    instructions: &'a [Instruction],
    local_types: &[DataType],
    culprit_instruction_with_index: (&Instruction, MemoryInstructionType, usize),
    split_type: SplitType,
    mut stack: Vec<StackValue>,
    scopes: &[Scope],
    mut deferred_splits: Vec<DeferredSplit<'a>>,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    if let Some(new_split) = handle_pre_split(
        base_name,
        culprit_instruction_with_index,
        instructions,
        local_types,
        base_split_count,
        &scopes,
        transformer,
    ) {
        deferred_splits.push(new_split);
    }
    match split_type {
        SplitType::Block => {
            transformer.emit_instruction("return".into(), None);
            let scope_end = index_of_scope_end(instructions)?;
            let mut sub_splits = handle_instructions(
                base_name,
                &instructions[scope_end..],
                local_types,
                stack,
                scopes.to_vec(),
                base_split_count + 1,
                transformer,
            )?;
            deferred_splits.append(&mut sub_splits);
        }
        SplitType::Normal => {
            transformer.emit_end_func();
        }
    }
    Ok(deferred_splits)
}

pub fn handle_pre_split<'a>(
    base_name: &str,
    culprit_instruction_with_index: (&Instruction, MemoryInstructionType, usize),
    instructions: &'a [Instruction],
    locals: &[DataType],
    split_count: usize,
    scopes: &[Scope],
    transformer: &mut WatEmitter,
) -> Option<DeferredSplit<'a>> {
    let (culprit, culprit_type, culprit_index) = culprit_instruction_with_index;
    let (pre_split, to_remove) = match culprit_type {
        MemoryInstructionType::Load { offset, .. } => {
            let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
            let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
            let offset_const = format!("i32.const {offset}");
            (
                vec![
                    (set_address, Some("Save address for load".into())),
                    ("local.get $utx".into(), None),
                    (get_address, None),
                    (offset_const, Some("Convert =offset to value".into())),
                    ("i32.add".into(), None),
                    ("i32.store".into(), None),
                ],
                1,
            )
        }
        MemoryInstructionType::Store { ty, offset } => {
            let ty = ty.as_str();
            let stack_juggler_local_name = format!("{ty}_{STACK_JUGGLER_NAME}");
            let set_value = format!("local.set ${stack_juggler_local_name}");
            let get_value = format!("local.get ${stack_juggler_local_name}");
            let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
            let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
            let store_data_type = format!(
                "{ty}.store offset={state_offset}",
                state_offset = transformer.state_base
            );
            let offset_const = format!("i32.const {offset}");
            (
                vec![
                    (set_value, Some("Save value for store".into())),
                    (set_address, Some("Save address for store".into())),
                    ("local.get $state".into(), None),
                    (get_value, None),
                    (
                        store_data_type,
                        Some(format!(
                            "First {n} bytes reserved for user defined state struct",
                            n = transformer.state_base
                        )),
                    ),
                    ("local.get $utx".into(), None),
                    (get_address, None),
                    (offset_const, Some("Convert =offset to value".into())),
                    ("i32.add".into(), None),
                    ("i32.store".into(), None),
                ],
                2,
            )
        }
    };

    for (pre_split_instr, annotation) in pre_split {
        transformer.emit_instruction(&pre_split_instr, annotation);
    }
    transformer.emit_instruction("local.get $utx".into(), Some("Save naddr = 1".into()));
    transformer.emit_instruction(&format!("i32.const 1"), None);
    transformer.emit_instruction("i32.store8 offset=35".into(), None);
    let stack_start = scopes.last().map(|scope| scope.stack_start).unwrap_or(0);
    let stack = &culprit.stack[..culprit.stack.len() - to_remove];

    transformer.emit_save_stack_and_locals(
        transformer.stack_base,
        stack,
        stack_start,
        false,
        locals,
    );

    // Check if a split has already been created for this instruction,
    // if so simply return that table index
    let existing_index = transformer
        .utx_function_names
        .iter()
        .position(|(address, _)| culprit_index == *address);
    let index = existing_index.unwrap_or(transformer.utx_function_names.len()) + 1;
    transformer.emit_instruction(
        &format!("i32.const {index}"),
        Some("Return index to next microtransaction".into()),
    );

    if let None = existing_index {
        let name = format!("{base_name}_{split_index}", split_index = split_count + 1);
        transformer
            .utx_function_names
            .push((culprit_index, name.clone()));
        Some(DeferredSplit {
            name,
            culprit_instruction: culprit_type,
            instructions,
            locals: locals.to_vec(),
            stack: stack.to_vec(),
            scopes: scopes.to_vec(),
        })
    } else {
        None
    }
}

pub fn handle_defered_split<'a>(
    mut deferred_split: DeferredSplit<'a>,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    setup_func(
        &deferred_split.name,
        deferred_split.instructions,
        &deferred_split.locals,
        transformer,
    );
    transformer.emit_restore_locals(
        &deferred_split.locals,
        transformer.stack_base,
        &deferred_split.stack,
    );
    if deferred_split.scopes.is_empty() {
        transformer.emit_restore_stack(
            transformer.stack_base,
            &deferred_split.stack,
            0,
            deferred_split.stack.len(),
        );
    } else {
        transformer.current_scope_level = 0;
        let mut curr_stack_base = 0;
        for scope in &deferred_split.scopes {
            match scope.ty {
                ScopeType::Block => {
                    transformer.emit_restore_stack(
                        transformer.stack_base,
                        &deferred_split.stack,
                        curr_stack_base,
                        scope.stack_start,
                    );
                    curr_stack_base = scope.stack_start;
                    let instruction = if let Some(name) = &scope.name {
                        // TODO - we need to enforce either (block) or `block end`
                        format!("(block ${name}")
                    } else {
                        "(block".into()
                    };
                    transformer.emit_instruction(&instruction, None);
                    transformer.current_scope_level += 1;
                }
            }
        }
        transformer.emit_restore_stack(
            transformer.stack_base,
            &deferred_split.stack,
            curr_stack_base,
            deferred_split.stack.len(),
        );
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
            let load_data_type = format!(
                "{ty}.load offset={state_base}",
                ty = ty.as_str(),
                state_base = transformer.state_base
            );
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
        deferred_split.instructions,
        &deferred_split.locals,
        deferred_split.stack,
        deferred_split.scopes,
        0,
        transformer,
    )
}

#[derive(Clone)]
pub struct DeferredSplit<'a> {
    name: String,
    culprit_instruction: MemoryInstructionType,
    instructions: &'a [Instruction<'a>],
    locals: Vec<DataType>,
    stack: Vec<StackValue>,
    scopes: Vec<Scope>,
}
