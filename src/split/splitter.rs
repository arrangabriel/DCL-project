use std::io::Write;

use wast::core::{Func, FuncKind, Instruction, ModuleField, ModuleKind};
use wast::Wat;

use crate::split::instruction_analysis::{
    BlockInstructionType, InstructionType, MemoryInstructionType, SplitType, StackEffect,
    StackValue,
};
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
    handle_instructions(
        &name,
        func_offset,
        instructions,
        0,
        Vec::default(),
        transformer,
    )
}

fn setup_func(name: &str, instructions: &[Instruction], transformer: &mut WatEmitter) {
    transformer.emit_utx_func_signature(name);
    transformer.utx_function_names.push(name.into());
    transformer.emit_locals_if_neccessary(instructions);
}

fn handle_instructions(
    name: &str,
    func_offset: usize,
    instructions: &[Instruction],
    instruction_offset: usize,
    mut stack: Vec<StackValue>,
    transformer: &mut WatEmitter,
) -> Result<(), &'static str> {
    let mut deferred_splits = Vec::new();
    for (i, instruction) in instructions.iter().enumerate() {
        let ty = InstructionType::from(instruction);
        if let Some(split_type) = ty.needs_split(&stack, transformer.skip_safe_splits)? {
            match split_type {
                SplitType::Normal(culprit_instruction) => {
                    let local_offset = i + 1;
                    let new_split = handle_normal_pre_split(
                        name,
                        func_offset,
                        culprit_instruction,
                        &instructions[local_offset..],
                        instruction_offset + local_offset,
                        deferred_splits.len(),
                        &mut stack,
                        transformer,
                    );
                    deferred_splits.push(new_split);
                    break;
                }
            }
        } else if let InstructionType::Benign(Some(ty)) = ty {
            match ty {
                BlockInstructionType::End => {}
                BlockInstructionType::Block => {}
                BlockInstructionType::Loop => {}
            }
        }
        StackEffect::from(instruction).update_stack(&mut stack)?;
        transformer.emit_instruction_from_function(func_offset, instruction_offset + i)?;
    }
    transformer.emit_end_func();
    for deferred_split in deferred_splits {
        handle_defered(deferred_split, transformer)?
    }
    Ok(())
}

fn handle_defered(
    mut deferred_split: DeferredSplit,
    transformer: &mut WatEmitter,
) -> Result<(), &'static str> {
    setup_func(
        &deferred_split.name,
        deferred_split.instructions,
        transformer,
    );
    transformer.emit_restore_stack(&deferred_split.stack);
    handle_post_split(&mut deferred_split, transformer);
    // This call recurses indirectly
    handle_instructions(
        &deferred_split.name,
        deferred_split.func_offset,
        deferred_split.instructions,
        deferred_split.instruction_offset,
        deferred_split.stack,
        transformer,
    )
}

fn handle_normal_pre_split<'a>(
    base_name: &str,
    func_offset: usize,
    culprit_instruction: MemoryInstructionType,
    instructions: &'a [Instruction],
    instruction_offset: usize,
    split_count: usize,
    stack: &mut Vec<StackValue>,
    transformer: &mut WatEmitter,
) -> DeferredSplit<'a> {
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
    transformer.emit_save_stack(&stack);
    transformer.emit_instruction(
        &format!(
            "i32.const {func_index}",
            func_index = transformer.utx_function_names.len()
        ),
        Some("Return index to next microtransaction".into()),
    );

    DeferredSplit {
        name: format!("{base_name}_{split_index}", split_index = split_count + 1),
        func_offset,
        culprit_instruction,
        instructions,
        instruction_offset,
        stack: stack.to_vec(),
    }
}

fn handle_post_split(deferred_split: &mut DeferredSplit, transformer: &mut WatEmitter) {
    let post_split = match deferred_split.culprit_instruction {
        MemoryInstructionType::Load { ty, .. } => {
            deferred_split.stack.push(StackValue { ty, is_safe: false });
            let load_data_type = format!("{}.load", ty.as_str());
            vec!["local.get $utx".into(), "i32.load".into(), load_data_type]
        }
        MemoryInstructionType::Store { ty, .. } => {
            let store_data_type = format!("{}.store", ty.as_str());
            let load_data_type = format!("{}.load", ty.as_str());
            vec![
                "local.get $utx".into(),
                "i32.load".into(),
                "local.get $state".into(),
                load_data_type,
                store_data_type,
            ]
        }
    };

    for post_split_instr in &post_split {
        transformer.emit_instruction(post_split_instr, None);
    }
}

struct DeferredSplit<'a> {
    name: String,
    func_offset: usize,
    culprit_instruction: MemoryInstructionType,
    instructions: &'a [Instruction<'a>],
    instruction_offset: usize,
    stack: Vec<StackValue>,
}
