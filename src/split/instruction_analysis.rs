use std::fmt::{Display, Formatter};

use wast::core::Instruction;
use wast::token::Index;
use Instruction::{
    DataDrop, Drop, ElemDrop, F32Load, F32Store, F64Load, F64Store, GlobalGet, GlobalSet, I32Add,
    I32Const, I32Eq, I32Load, I32Mul, I32Store, I32Store8, I32WrapI64, I64Add, I64Const, I64Eq,
    I64Load, I64Mul, I64Store, I64Store8, LocalGet, MemoryCopy, MemoryDiscard, MemoryFill,
    MemoryGrow, MemoryInit, MemorySize, TableCopy, TableFill, TableGet, TableGrow, TableInit,
    TableSet, TableSize,
};

use DataType::*;
use InstructionType::{Benign, Memory};

use crate::split::transform::Scope;
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
}

impl From<&Instruction<'_>> for StackEffect {
    fn from(value: &Instruction) -> Self {
        match value {
            Instruction::Return // The return might have to be handled with care
            | Instruction::End(_)
            | Instruction::Block(_)
            | Instruction::Br(_) => StackEffect::new(0, None, false, false),
            LocalGet(index) => match index {
                Index::Num(_, _) => panic!("Unsupported num index"),
                Index::Id(id) => StackEffect::new(0, Some(I32), name_is_param(id.name()), true),
            },
            I64Load(_) => StackEffect::new(1, Some(I64), false, false),
            I64Const(_) => StackEffect::new(0, Some(I64), false, false),
            I32WrapI64 | I32Load(_) => StackEffect::new(1, Some(I32), false, true),
            I32Const(_) => StackEffect::new(0, Some(I32), false, false),
            I32Mul | I32Add | I32Eq => StackEffect::new(2, Some(I32), false, false),
            I64Mul | I64Add | I64Eq => StackEffect::new(2, Some(I64), false, false),
            I32Store(_) | I32Store8(_) => StackEffect::new(2, None, false, false),
            Drop => StackEffect::new(1, None, false, false),
            Instruction::F64Const(_) => StackEffect::new(0, Some(F64), false, false),
            Instruction::F32Const(_) => StackEffect::new(0, Some(F32), false, false),
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
    Block(Option<String>),
}

impl InstructionType {
    pub fn needs_split(
        &self,
        stack: &[StackValue],
        scopes: &[Scope],
        skip_safe_splits: bool,
    ) -> Result<Option<(SplitType, MemoryInstructionType)>, &'static str> {
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
        let split_type = ty.map(|&ty| {
            (
                if scopes.is_empty() {
                    SplitType::Normal
                } else {
                    SplitType::Block
                },
                ty,
            )
        });
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
            let instruction_type = match value {
                // support if and else at a later date
                Instruction::Block(id) => Some(BlockInstructionType::Block(
                    id.label.map(|id| id.name().into()),
                )),
                Instruction::End(_) => Some(BlockInstructionType::End),
                _ => None,
            };
            Benign(instruction_type)
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
    Normal,
    Block,
}

/// To be used at some point inside of a scope
pub fn index_of_scope_end(
    instructions_with_index: &[(&Instruction, usize)],
) -> Result<usize, &'static str> {
    let mut scope_level = 1;
    for (i, &(instruction, _)) in instructions_with_index.iter().enumerate() {
        if let Benign(Some(block_instruction_type)) = InstructionType::from(instruction) {
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
