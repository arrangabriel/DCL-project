use crate::chop_up::function::index_of_scope_end;
use crate::chop_up::instruction::{DataType, MemoryInstructionType};
use crate::chop_up::transform::{handle_instructions, setup_func};
use crate::chop_up::emit::WatEmitter;
use crate::chop_up::instruction_stream::{Instruction, Scope, ScopeType, StackValue};
#[allow(unused_imports)] // This is due to a bug in my linter...
use crate::chop_up::constants::{ADDRESS_LOCAL_NAME, STACK_JUGGLER_NAME};

pub fn setup_split<'a>(
    base_name: &str,
    split_count: usize,
    instructions: &'a [Instruction],
    locals: &[DataType],
    culprit_instruction_with_index: (&Instruction, MemoryInstructionType, usize),
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    let mut deferred_splits = Vec::default();
    if let Some(new_split) = handle_pre_split(
        base_name,
        culprit_instruction_with_index,
        instructions,
        locals,
        split_count,
        transformer,
    ) {
        deferred_splits.push(new_split);
    }
    if culprit_instruction_with_index.0.scopes.is_empty() {
        transformer.emit_end_func();
    } else {
        transformer.emit_instruction("return", None);
        let scope_end = index_of_scope_end(instructions)?;
        let mut sub_splits = handle_instructions(
            base_name,
            &instructions[scope_end..],
            locals,
            split_count + 1,
            transformer,
        )?;
        deferred_splits.append(&mut sub_splits);
    }
    Ok(deferred_splits)
}

pub fn handle_pre_split<'a>(
    base_name: &str,
    culprit_instruction_with_index: (&Instruction, MemoryInstructionType, usize),
    instructions: &'a [Instruction],
    locals: &[DataType],
    split_count: usize,
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
        MemoryInstructionType::Store { ty, offset, subtype: _} => {
            // TODO - do we need to take special care if subtype is not None?
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
    transformer.emit_instruction("local.get $utx", Some("Save naddr = 1".into()));
    transformer.emit_instruction("i32.const 1", None);
    transformer.emit_instruction("i32.store8 offset=35", None);

    let stack_start = culprit
        .scopes
        .last()
        .map(|scope| scope.stack_start)
        .unwrap_or(0);
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

    if existing_index.is_none() {
        let name = format!("{base_name}_{split_index}", split_index = split_count + 1);
        transformer
            .utx_function_names
            .push((culprit_index, name.clone()));
        Some(DeferredSplit {
            name,
            culprit_type,
            instructions,
            locals: locals.to_vec(),
            saved_stack: stack.to_vec(),
            scopes: culprit.scopes.to_vec(),
        })
    } else {
        None
    }
}

pub fn handle_deferred_split<'a>(
    deferred_split: DeferredSplit<'a>,
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
        &deferred_split.saved_stack,
    );
    if deferred_split.scopes.is_empty() {
        transformer.emit_restore_stack(
            &deferred_split.saved_stack,
            0,
            deferred_split.saved_stack.len(),
        );
    } else {
        transformer.current_scope_level = 0;
        let mut curr_stack_base = 0;
        for scope in &deferred_split.scopes {
            match scope.ty {
                ScopeType::Block => {
                    transformer.emit_restore_stack(
                        &deferred_split.saved_stack,
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
            &deferred_split.saved_stack,
            curr_stack_base,
            deferred_split.saved_stack.len(),
        );
    }
    let post_split: Vec<(String, Option<String>)> = match deferred_split.culprit_type {
        // TODO - this needs to emit the correct load type (e.g. i32.load16_u ...)
        MemoryInstructionType::Load { ty, subtype, .. } => {
            let subtype_str = subtype.map(|ty| ty.as_str()).unwrap_or("");
            let load_data_type = format!("{}.load{subtype_str}", ty.as_str());
            vec![
                ("local.get $utx".into(), Some("Restore load address".into())),
                ("i32.load".into(), None),
                (load_data_type, None),
            ]
        }
        MemoryInstructionType::Store { ty, subtype, .. } => {
            let subtype_str = subtype.map(|ty| ty.as_str()).unwrap_or("");
            let store_data_type = format!("{}.store{subtype_str}", ty.as_str());
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
        0,
        transformer,
    )
}

#[derive(Clone)]
pub struct DeferredSplit<'a> {
    name: String,
    culprit_type: MemoryInstructionType,
    instructions: &'a [Instruction<'a>],
    locals: Vec<DataType>,
    saved_stack: Vec<StackValue>,
    scopes: Vec<Scope>,
}
