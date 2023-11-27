use std::cmp::Ordering;
use wast::core::{Func, FuncKind};
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::chop_up::instruction::{
    BenignInstructionType, BlockInstructionType, DataType, InstructionType,
};
use crate::chop_up::instruction_stream::{Instruction, Scope, ScopeType, StackEffect};

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
            None => gen_random_func_name(),
            Some(func_name) => {
                func_name.into()
            },
        };


        let (wast_instructions, local_types) =
            if let FuncKind::Inline { expression, locals } = &func.kind {
                let local_types = locals
                    .iter()
                    .map(|local| local.ty.into())
                    .collect::<Vec<DataType>>();
                Ok((expression.instrs.iter().as_slice(), local_types))
            } else {
                Err("FuncKind is not inline")
            }?;

        let function_line_index = get_line_index_from_offset(lines, func.span.offset());
        let signature = lines[function_line_index].trim();

        let function_member_base_line_index = function_line_index + 1;
        let instruction_base_line_index = function_member_base_line_index
            + lines[function_member_base_line_index..]
                .iter()
                .take_while(|line| line.contains("(local"))
                .count();

        let mut instructions_with_raw_text = Vec::default();
        for (instruction, &raw_string) in wast_instructions
            .iter()
            .zip(
                &lines[instruction_base_line_index
                    ..instruction_base_line_index + wast_instructions.len()],
            ) {
            instructions_with_raw_text.push((instruction, raw_string.trim()))
        }

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
                        index: instruction_base_line_index + i
                    }
                }).collect()
            })
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

/// To be used at some point inside of a scope
pub fn index_of_scope_end(instructions: &[Instruction]) -> Result<usize, &'static str> {
    let mut scope_level = 1;
    for (i, instruction_with_text) in instructions.iter().enumerate() {
        if let InstructionType::Benign(BenignInstructionType::Block(block_instruction_type)) =
            InstructionType::from(instruction_with_text)
        {
            scope_level += match block_instruction_type {
                BlockInstructionType::End => -1,
                BlockInstructionType::Block(_) => 1,
            };

            match scope_level.cmp(&0) {
                Ordering::Equal => return Ok(i),
                Ordering::Less => return Err("Unbalanced scope delimiters"),
                Ordering::Greater => {}
            }
        }
    }
    Err("Unbalanced scope delimiters")
}

fn get_line_index_from_offset<'a>(lines: &'a [&'a str], offset: usize) -> usize {
    let total_len = lines.iter().map(|l| l.len() + 1).sum();
    assert!(offset < total_len, "Offset provided was out of bounds");
    let mut line_end = 0;
    for (i, line) in lines.iter().enumerate() {
        line_end += line.len() + 1;
        if offset < line_end {
            return i;
        }
    }
    unreachable!()
}

pub fn get_line_from_offset<'a>(lines: &'a [&'a str], offset: usize) -> &'a str {
    lines[get_line_index_from_offset(lines, offset)]
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
