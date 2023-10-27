use wast::core::Instruction;
use wast::core::Instruction::{
    DataDrop, Drop, ElemDrop, F32Load, F32Store, F64Load, F64Store, GlobalGet, GlobalSet, I32Load,
    I32Store, I32Store8, I64Load, I64Store, I64Store8, MemoryCopy, MemoryDiscard, MemoryFill,
    MemoryGrow, MemoryInit, MemorySize, TableCopy, TableFill, TableGet, TableGrow, TableInit,
    TableSet, TableSize,
};
use wast::token::Index;
use Instruction::{I32Add, I32Const, I32Mul, I32WrapI64};

use DataType::*;

use crate::ast_parsing::StackEffect::{Add, Binary, Remove, RemoveTwo, Unary};

#[derive(PartialEq)]
pub enum InstructionType {
    Memory(MemoryInstructionType),
    Benign,
}

#[derive(PartialEq)]
pub enum MemoryInstructionType {
    Load { ty: DataType, offset: u64 },
    Store { ty: DataType, offset: u64 },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DataType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(PartialEq)]
pub enum StackEffect {
    Unary(DataType),
    Binary(DataType),
    Add(DataType),
    Remove,
    RemoveTwo,
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

pub fn get_instruction_effect(instruction: &Instruction) -> StackEffect {
    match instruction {
        Instruction::LocalGet(index) => match index {
            Index::Num(_, _) => panic!("Unsupported num index"),
            Index::Id(_) => Add(I32),
        },
        I64Load(_) => Binary(I64),
        I32WrapI64 => Unary(I32),
        I32Const(_) => Add(I32),
        I32Mul | I32Add | I32Load(_) => Binary(I32),
        I32Store(_) | I32Store8(_) => RemoveTwo,
        Drop => Remove,
        _ => panic!("Unsupported instruction read - {:?}", instruction),
    }
}

pub fn get_instruction_type(instruction: &Instruction) -> InstructionType {
    if let Some((ty, offset)) = type_from_load(instruction) {
        InstructionType::Memory(MemoryInstructionType::Load { ty, offset })
    } else if let Some((ty, offset)) = type_from_store(instruction) {
        InstructionType::Memory(MemoryInstructionType::Store { ty, offset })
    } else if is_other_memory_instruction(instruction) {
        panic!("Unsupported instruction read - {:?}", instruction)
    } else {
        InstructionType::Benign
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
