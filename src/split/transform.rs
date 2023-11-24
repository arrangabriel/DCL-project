use std::io::Write;

use crate::split::function_analysis;
use wast::core::{ModuleField, ModuleKind};
use wast::Wat;

use crate::split::function_analysis::{Function, StackEffect, StackValue};
use crate::split::instruction_types::{
    BenignInstructionType, BlockInstructionType, DataType, Instruction, InstructionType,
};
use crate::split::split::{handle_defered_split, setup_split, DeferredSplit};
use crate::split::utils::MODULE_MEMBER_INDENT;
use crate::split::wat_emitter::WatEmitter;

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
            ModuleField::Func(func) => functions.push(Function::new(func, lines)?),
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
            .drain(..deferred_splits.len())
            .flat_map(|split| handle_defered_split(split, &mut transformer).unwrap())
            .collect();
    }

    for saved_offset in saved_offsets {
        let line = function_analysis::get_line_from_offset(lines, saved_offset);
        let extra_parens = line.chars().fold(0, |v, c| {
            v + match c {
                '(' => -1,
                ')' => 1,
                _ => 0,
            }
        }) as usize;
        transformer.writeln(
            &line[..line.len() - extra_parens].trim(),
            MODULE_MEMBER_INDENT,
        );
    }

    transformer.emit_end_module();
    Ok(())
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
        Vec::default(),
        Vec::default(),
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
    mut stack: Vec<StackValue>,
    mut scopes: Vec<Scope>,
    split_count: usize,
    transformer: &mut WatEmitter,
) -> Result<Vec<DeferredSplit<'a>>, &'static str> {
    let deferred_splits: Vec<DeferredSplit> = Vec::default();
    transformer.current_scope_level = scopes.len();
    for (i, instruction) in instructions.iter().enumerate() {
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
                        locals,
                        (ty, instruction.index),
                        split_type,
                        stack,
                        &scopes,
                        deferred_splits,
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
                                transformer.emit_save_stack_and_locals(
                                    transformer.stack_base,
                                    &stack,
                                    prev_stack_start,
                                    true,
                                    locals,
                                );
                                scopes.push(Scope {
                                    ty: ScopeType::Block,
                                    name,
                                    stack_start,
                                });
                                transformer.emit_instruction(instruction.raw_text, None);
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
                    BenignInstructionType::IndexedLocal(ty, index) => {
                        // After changins function signatures:
                        // tx, state -> tx, utx, state
                        // all locals after the first have to be incremented by one
                        let instruction_str =
                            format!("local.{ty_str} {index}", ty_str = ty.as_str());
                        transformer.emit_instruction(&instruction_str, None);
                        StackEffect::from_instruction(instruction, locals)
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
        StackEffect::from_instruction(instruction, locals).update_stack(&mut stack)?;
        transformer.emit_instruction(instruction.raw_text, None);
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
