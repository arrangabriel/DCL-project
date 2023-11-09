use wast::core::Instruction;
use wast::core::Instruction::{
    DataDrop, Drop, ElemDrop, F32Load, F32Store, F64Load, F64Store, GlobalGet, GlobalSet, I32Load,
    I32Store, I32Store8, I64Load, I64Store, I64Store8, MemoryCopy, MemoryDiscard, MemoryFill,
    MemoryGrow, MemoryInit, MemorySize, TableCopy, TableFill, TableGet, TableGrow, TableInit,
    TableSet, TableSize,
};
use wast::token::Index;
use BlockInstructionType::Block;
use Instruction::{I32Add, I32Const, I32Mul, I32WrapI64};

use crate::split::instruction_analysis::BlockInstructionType::{End, Loop};
use DataType::*;
use InstructionType::{Benign, Memory};

use crate::split::utils::name_is_param;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DataType {
    I32,
    I64,
    F32,
    F64,
}

impl DataType {
    pub fn as_str(&self) -> &str {
        match self {
            I32 => "i32",
            I64 => "i64",
            F32 => "f32",
            F64 => "f64",
        }
    }

    pub fn size(&self) -> usize {
        match self {
            F32 | I32 => 4,
            I64 | F64 => 8,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MemoryInstructionType {
    Load { ty: DataType, offset: u64 },
    Store { ty: DataType, offset: u64 },
}

#[derive(Copy, Clone, Debug)]
pub struct StackValue {
    pub ty: DataType,
    pub is_safe: bool,
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
}

impl From<&Instruction<'_>> for StackEffect {
    fn from(value: &Instruction) -> Self {
        match value {
            Instruction::LocalGet(index) => match index {
                Index::Num(_, _) => panic!("Unsupported num index"),
                Index::Id(id) => StackEffect::new(0, Some(I32), name_is_param(id.name()), true),
            },
            I64Load(_) => StackEffect::new(1, Some(I64), false, false),
            I32WrapI64 => StackEffect::new(1, Some(I32), false, true),
            I32Const(_) => StackEffect::new(0, Some(I32), false, false),
            I32Mul | I32Add | I32Load(_) => StackEffect::new(2, Some(I32), false, false),
            I32Store(_) | I32Store8(_) => StackEffect::new(2, None, false, false),
            Drop => StackEffect::new(1, None, false, false),
            _ => panic!("Unsupported instruction read when producing StackEffect - {value:?}"),
        }
    }
}

#[derive(PartialEq)]
pub enum InstructionType {
    Memory(MemoryInstructionType),
    Benign(Option<BlockInstructionType>),
}

#[derive(PartialEq)]
pub enum BlockInstructionType {
    End,
    Block,
    Loop,
}

impl InstructionType {
    pub fn needs_split(
        &self,
        stack: &[StackValue],
        skip_safe_splits: bool,
    ) -> Result<Option<SplitType>, &'static str> {
        let ty = match self {
            Memory(ty) => match ty {
                MemoryInstructionType::Load { .. } => {
                    let last_is_safe = stack
                        .last()
                        .ok_or("Load with empty stack - program is malformed")?
                        .is_safe;
                    if last_is_safe && skip_safe_splits {
                        None
                    } else {
                        Some(ty)
                    }
                }
                MemoryInstructionType::Store { .. } => Some(ty),
            },
            Benign(_) => None,
        };
        let split_type = ty.map(|&ty| SplitType::Normal(ty));
        Ok(split_type)
    }
}

impl From<&Instruction<'_>> for InstructionType {
    fn from(value: &Instruction) -> Self {
        if let Some((ty, offset)) = type_from_load(value) {
            Memory(MemoryInstructionType::Load { ty, offset })
        } else if let Some((ty, offset)) = type_from_store(value) {
            Memory(MemoryInstructionType::Store { ty, offset })
        } else if is_other_memory_instruction(value) {
            panic!("Unsupported instruction read when producing InstructionType - {value:?}")
        } else {
            match value {
                // support if and else at a later date
                Instruction::Block(_) => Benign(Some(Block)),
                Instruction::Loop(_) => Benign(Some(Loop)),
                Instruction::End(_) => Benign(Some(End)),
                _ => Benign(None),
            }
        }
    }
}

fn type_from_load(instruction: &Instruction) -> Option<(DataType, u64)> {
    match instruction {
        I32Load(arg) => Some((I32, arg.offset)),
        I64Load(arg) => Some((I64, arg.offset)),
        F32Load(arg) => Some((F32, arg.offset)),
        F64Load(arg) => Some((F64, arg.offset)),
        _ => None,
    }
}

fn type_from_store(instruction: &Instruction) -> Option<(DataType, u64)> {
    match instruction {
        I32Store(arg) | I32Store8(arg) => Some((I32, arg.offset)),
        I64Store(arg) | I64Store8(arg) => Some((I64, arg.offset)),
        F32Store(arg) => Some((F32, arg.offset)),
        F64Store(arg) => Some((F64, arg.offset)),
        _ => None,
    }
}

fn is_other_memory_instruction(instruction: &Instruction) -> bool {
    match instruction {
        GlobalGet(_) | GlobalSet(_) | TableGet(_) | TableSet(_) | MemorySize(_) | MemoryGrow(_)
        | MemoryInit(_) | MemoryCopy(_) | MemoryFill(_) | MemoryDiscard(_) | DataDrop(_)
        | ElemDrop(_) | TableInit(_) | TableCopy(_) | TableFill(_) | TableSize(_)
        | TableGrow(_) => true,
        _ => false,
    }
}

pub enum SplitType {
    Normal(MemoryInstructionType),
}
