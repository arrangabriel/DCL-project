use std::fmt::{Display, Formatter};

use wast::core::Instruction::{I32Sub, LocalSet};
use wast::core::{Func, FuncKind, Instruction as WastInstruction};
use wast::token::Index;
use WastInstruction::{
    Block, Br, BrIf, Drop, End, F32Const, F32Gt, F64Const, F64Gt, I32Add, I32Const, I32Eq, I32Eqz,
    I32GtS, I32GtU, I32Load, I32Load16u, I32LtS, I32LtU, I32Mul, I32Ne, I32Shl, I32Store,
    I32Store8, I32WrapI64, I32Xor, I64Add, I64Const, I64Eq, I64ExtendI32U, I64GtS, I64GtU, I64Load,
    I64LtS, I64LtU, I64Mul, I64Ne, I64Sub, I64Xor, LocalGet, LocalTee, Return,
};

use crate::split::instruction_types::InstructionType::Benign;
use crate::split::instruction_types::{
    BenignInstructionType, BlockInstructionType, DataType, Instruction, InstructionType,
};

use crate::split::utils::{
    gen_random_func_name, index_is_param, name_is_param, IGNORE_FUNC_PREFIX,
};

fn type_and_safety_from_param(index: &Index, local_types: &[DataType]) -> (DataType, bool) {
    match index {
        Index::Num(index, _) => {
            let index = *index as usize;
            let safe = index_is_param(index);
            let mut utx_locals = Vec::default();
            utx_locals.extend_from_slice(&UTX_LOCALS);
            utx_locals.extend_from_slice(local_types);
            let ty = *utx_locals
                .get(index)
                .expect("Indexed get to locals should use in bounds index");
            (ty, safe)
        }
        // TODO!!
        // All id'd instructions being I32 is not correct, this needs to change
        // luckily compiled code usually uses indexes, not ids.
        Index::Id(id) => (DataType::I32, name_is_param(id.name())),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StackValue {
    pub ty: DataType,
    pub is_safe: bool,
}

impl Display for StackValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let safe_string = if self.is_safe { " - safe" } else { "" };
        write!(f, "({:?}{safe_string})", self.ty)
    }
}

pub struct StackEffect {
    pub remove_n: usize,
    pub add: Option<StackValue>,
    pub preserves_safety: bool,
}

impl StackEffect {
    fn new(remove_n: usize, add: Option<DataType>, is_safe: bool, preserves_safety: bool) -> Self {
        Self {
            remove_n,
            add: add.map(|ty| StackValue { ty, is_safe }),
            preserves_safety,
        }
    }

    pub fn update_stack(&self, stack: &mut Vec<StackValue>) -> Result<(), &'static str> {
        let mut is_safe = false;
        for _ in 0..self.remove_n {
            let stack_value = stack
                .pop()
                .ok_or("Unbalanced stack - input program is malformed")?;
            is_safe |= self.preserves_safety && self.remove_n == 1 && stack_value.is_safe;
        }
        if let Some(mut stack_value) = self.add {
            stack_value.is_safe |= is_safe;
            stack.push(stack_value);
        }
        Ok(())
    }

    pub fn from_instruction(instruction: &Instruction, local_types: &[DataType]) -> Self {
        match instruction.instr {
            Return // The return might have to be handled with care
            | End(_) | Block(_) | Br(_) => StackEffect::new(0, None, false, false),
            LocalGet(index) => {
                let (ty, is_safe) = type_and_safety_from_param(index, local_types);
                StackEffect::new(0, Some(ty), is_safe, true)
            }
            LocalTee(_) => StackEffect::new(0, None, false, false),
            I64Load(_) | I64ExtendI32U => StackEffect::new(1, Some(DataType::I64), false, false),
            I64Const(_) => StackEffect::new(0, Some(DataType::I64), false, false),
            I32WrapI64 | I32Load(_) | I32Load16u(_) | I32Eqz => StackEffect::new(1, Some(DataType::I32), false, true),
            I32Const(_) => StackEffect::new(0, Some(DataType::I32), false, false),
            I32Mul | I32Add | I32Sub | I32Eq | F64Gt | F32Gt |
            I32GtU | I32GtS | I64GtU | I64GtS | I32LtU |
            I32LtS | I64LtU | I64LtS | I64Eq | I32Ne | I64Ne |
            I32Shl | I32Xor => StackEffect::new(2, Some(DataType::I32), false, false),
            I64Mul | I64Add | I64Xor | I64Sub => StackEffect::new(2, Some(DataType::I64), false, false),
            I32Store(_) | I32Store8(_) => StackEffect::new(2, None, false, false),
            Drop | BrIf(_) | LocalSet(_) => StackEffect::new(1, None, false, false),
            F64Const(_) => StackEffect::new(0, Some(DataType::F64), false, false),
            F32Const(_) => StackEffect::new(0, Some(DataType::F32), false, false),
            _ => panic!("Unsupported instruction read when producing StackEffect - {:?}", instruction.instr),
        }
    }
}

const UTX_LOCALS: [DataType; 3] = [DataType::I32, DataType::I32, DataType::I32];

pub enum SplitType {
    Normal,
    Block,
}

/// To be used at some point inside of a scope
pub fn index_of_scope_end(instructions: &[Instruction]) -> Result<usize, &'static str> {
    let mut scope_level = 1;
    for (i, instruction_with_text) in instructions.iter().enumerate() {
        if let Benign(BenignInstructionType::Block(block_instruction_type)) =
            InstructionType::from(instruction_with_text)
        {
            scope_level += match block_instruction_type {
                BlockInstructionType::End => -1,
                BlockInstructionType::Block(_) => 1,
            };
            if scope_level == 0 {
                return Ok(i);
            } else if scope_level < 0 {
                return Err("Unbalanced scope delimiters");
            }
        }
    }
    Err("Unbalanced scope delimiters")
}

pub struct Function<'a> {
    pub(crate) name: String,
    pub signature: &'a str,
    pub local_types: Vec<DataType>,
    pub instructions: Vec<Instruction<'a>>,
}

impl<'a> Function<'a> {
    pub fn new(func: &'a Func, lines: &'a [&'a str]) -> Result<Self, &'static str> {
        let name = match func.id.map(|id| id.name()) {
            None => gen_random_func_name(),
            Some(func_name) => func_name.into(),
        };
        let (instructions, local_types) =
            if let FuncKind::Inline { expression, locals } = &func.kind {
                let local_types = locals
                    .iter()
                    .map(|local| DataType::from(local.ty))
                    .collect();
                Ok((expression.instrs.iter().as_slice(), local_types))
            } else {
                Err("FuncKind is not inline")
            }?;
        let function_index = get_line_index_from_offset(&lines, func.span.offset());
        let signature = lines[function_index].trim();
        let function_member_base_index = function_index + 1;
        let instruction_base_index = function_member_base_index
            + lines[function_member_base_index..]
                .iter()
                .take_while(|line| line.contains("(local"))
                .count();
        let instructions: Vec<Instruction> = instructions
            .iter()
            .zip(&lines[instruction_base_index..instruction_base_index + instructions.len()])
            .enumerate()
            .map(|(i, (instruction, raw_text))| {
                Instruction::new(instruction, raw_text, instruction_base_index + i)
            })
            .collect();
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
