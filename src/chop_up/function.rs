use rand::distributions::Alphanumeric;
use rand::Rng;
use wast::core::{Func, FuncKind};
use wast::core::Instruction as WastInstruction;
use wast::token::Index;

use crate::chop_up::instruction::{
    BenignInstructionType, BlockInstructionType, DataType, InstructionType,
};
use crate::chop_up::instruction_stream::{index_is_param, Instruction, Scope, ScopeType, StackEffect};
use crate::chop_up::utils;
use crate::chop_up::utils::{count_parens, UTX_LOCALS};

pub struct Function<'a> {
    pub name: String,
    pub signature: &'a str,
    pub local_types: Vec<DataType>,
    pub instructions: Vec<Instruction<'a>>,
}

impl<'a> Function<'a> {
    // TODO - this function is doing way to much work
    pub fn new(func: &'a Func, lines: &'a [&str]) -> Result<Self, &'static str> {
        let name = match func.id.map(|id| id.name()) {
            Some(func_name) => func_name.into(),
            None => gen_random_func_name(),
        };

        let (wast_instructions, mut local_types) =
            if let FuncKind::Inline { expression, locals } = &func.kind {
                let local_types = locals
                    .iter()
                    .map(|local| local.ty.into())
                    .collect::<Vec<DataType>>();
                (expression.instrs.iter().as_slice(), local_types)
            } else { return Err("FuncKind is not inline"); };

        let function_line_index = utils::get_line_index_from_offset(lines, func.span.offset());
        let signature = lines[function_line_index].trim();

        let function_member_base_line_index = function_line_index + 1;
        let instruction_base_line_index = function_member_base_line_index
            + lines[function_member_base_line_index..]
            .iter()
            .take_while(|line| line.contains("(local"))
            .count();

        let mut instructions_with_raw_text = Vec::new();
        let mut remapped_locals: Vec<(u32, u32)> = Vec::new();
        for (instruction, &raw_string) in wast_instructions
            .iter()
            .zip(
                &lines[instruction_base_line_index
                    ..instruction_base_line_index + wast_instructions.len()],
            ) {
            let mut instruction_string = raw_string.trim().to_string();
            let mut paren_imbalance = count_parens(raw_string);
            if matches!(instruction, WastInstruction::End(_)) {
                paren_imbalance -= 1;
            }
            if paren_imbalance > 0 {
                let actual_len = instruction_string.len() - paren_imbalance as usize;
                instruction_string = instruction_string[..actual_len].trim().to_string();
            }
            // Here there is a possibly scary assumption made.
            // That the compiler will not use named arguments to local-instructions
            // To fix we must also handle the [Index::Id] case
            match instruction {
                WastInstruction::LocalSet(Index::Num(i, _)) | WastInstruction::LocalTee(Index::Num(i, _)) => {
                    if index_is_param(*i) {
                        let new_name = if let Some((_, new_name)) = remapped_locals.iter().find(|(param, _)| param.eq(i)) {
                            *new_name
                        } else {
                            let new_name = (UTX_LOCALS.len() + local_types.len() + remapped_locals.len()) as u32;
                            remapped_locals.push((*i, new_name));
                            new_name
                        };
                        instruction_string = format!(
                            "{base_instruction} {new_name}",
                            base_instruction = &instruction_string[..instruction_string.len() - 2]
                        );
                    }
                }
                WastInstruction::LocalGet(Index::Num(i, _)) => {
                    if index_is_param(*i) {
                        if let Some(new_name) = remapped_locals.iter().find(|(param, _)| param.eq(i)).map(|(_, new_name)| *new_name) {
                            instruction_string = format!(
                                "{base_instruction} {new_name}",
                                base_instruction = &instruction_string[..instruction_string.len() - 2]
                            );
                        }
                    }
                }
                _ => {}
            }
            instructions_with_raw_text.push((instruction, instruction_string))
        }

        for _ in remapped_locals {
            local_types.push(DataType::I32)
        }

        // No preprocessing is needed in this case...
        if name.starts_with(IGNORE_FUNC_PREFIX) {
            return Ok(Self {
                name,
                signature,
                local_types,
                instructions: instructions_with_raw_text.into_iter().enumerate().map(|(i, (instr, raw_text))| {
                    Instruction {
                        instr,
                        raw_text,
                        scopes: Vec::default(),
                        stack: Vec::default(),
                        index: instruction_base_line_index + i,
                    }
                }).collect(),
            });
        }

        let mut instructions_with_text_and_stack = Vec::default();
        let mut current_stack_state = Vec::default();
        for (instruction, raw_string) in instructions_with_raw_text {
            instructions_with_text_and_stack.push((instruction, raw_string, current_stack_state.to_vec()));
            StackEffect::from_wast_instruction(instruction, &local_types)
                .update_stack(&mut current_stack_state)?;
        }

        let mut instructions_with_stack_and_scope = Vec::default();
        let mut current_scopes = Vec::default();
        for (instruction, text, stack) in instructions_with_text_and_stack {
            if let InstructionType::Benign(BenignInstructionType::Block(ty)) =
                InstructionType::from(instruction)
            {
                match ty {
                    BlockInstructionType::Block(name) => {
                        current_scopes.push(Scope {
                            ty: ScopeType::Block,
                            name,
                            stack_start: stack.len(),
                        });
                    }
                    BlockInstructionType::End => {
                        let scope = current_scopes
                            .pop()
                            .ok_or("Unbalanced scopes - tried to remove top-level scope")?;
                        match scope.ty {
                            ScopeType::Block => {
                                // TODO
                                // Slice off popped scope stack
                                // returns in particular
                            }
                        }
                    }
                }
            }
            instructions_with_stack_and_scope.push((instruction, text, stack, current_scopes.to_vec()))
        }

        let mut instructions = Vec::default();
        for (i, (instruction, raw_text, stack, scopes)) in instructions_with_stack_and_scope
            .into_iter()
            .enumerate()
        {
            instructions.push(Instruction::new(
                instruction,
                raw_text,
                instruction_base_line_index + i,
                stack,
                scopes,
            ));
        }

        Ok(Function {
            name,
            local_types,
            signature,
            instructions,
        })
    }

    pub fn ignore(&self) -> bool {
        self.name.starts_with(IGNORE_FUNC_PREFIX)
    }
}

fn gen_random_func_name() -> String {
    let rand_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();
    format!("funcid_{rand_id}")
}

const IGNORE_FUNC_PREFIX: &str = "__";
