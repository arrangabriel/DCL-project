use wast::core::Instruction;
use wast::core::Instruction::{
    DataDrop, ElemDrop, F32Load, F32Store, F64Load, F64Store, GlobalGet, GlobalSet, I32Load,
    I32Load16s, I32Load16u, I32Load8s, I32Load8u, I32Store, I32Store16, I32Store8, I64Load,
    I64Load16s, I64Load16u, I64Load32s, I64Load32u, I64Load8s, I64Load8u, I64Store, I64Store16,
    I64Store32, I64Store8, MemoryCopy, MemoryDiscard, MemoryFill, MemoryGrow, MemoryInit,
    MemorySize, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableSet, TableSize,
};

#[derive(PartialEq)]
pub enum InstructionType {
    Memory(MemoryInstructionType),
    Benign,
}

#[derive(PartialEq)]
pub enum MemoryInstructionType {
    Load(DataType),
    Store(DataType),
    OtherMem,
}

#[derive(PartialEq)]
pub enum DataType {
    I32,
    I64,
    F32,
    F64,
}

impl DataType {
    pub fn as_str(&self) -> &str {
        match self {
            DataType::I32 => "i32",
            DataType::I64 => "i64",
            DataType::F32 => "f32",
            DataType::F64 => "f64",
        }
    }
}

impl InstructionType {
    pub fn is_mem_access_instruction(&self) -> bool {
        return self != &InstructionType::Benign;
    }
}

pub fn get_instruction_type(instruction: &Instruction) -> InstructionType {
    if let Some(data_type) = type_from_load(instruction) {
        InstructionType::Memory(MemoryInstructionType::Load(data_type))
    } else if let Some(data_type) = type_from_store(instruction) {
        InstructionType::Memory(MemoryInstructionType::Store(data_type))
    } else if is_other_memory_instruction(instruction) {
        InstructionType::Memory(MemoryInstructionType::OtherMem)
    } else {
        InstructionType::Benign
    }
}

fn type_from_load(instruction: &Instruction) -> Option<DataType> {
    match instruction {
        I32Load(_) | I32Load8s(_) | I32Load8u(_) | I32Load16s(_) | I32Load16u(_) => {
            Some(DataType::I32)
        }
        I64Load(_) | I64Load8s(_) | I64Load8u(_) | I64Load16s(_) | I64Load16u(_)
        | I64Load32s(_) | I64Load32u(_) => Some(DataType::I64),
        F32Load(_) => Some(DataType::F32),
        F64Load(_) => Some(DataType::F64),
        _ => None,
    }
}

fn type_from_store(instruction: &Instruction) -> Option<DataType> {
    match instruction {
        I32Store(_) | I32Store8(_) | I32Store16(_) => Some(DataType::I32),
        I64Store(_) | I64Store8(_) | I64Store16(_) | I64Store32(_) => Some(DataType::I64),
        F32Store(_) => Some(DataType::F32),
        F64Store(_) => Some(DataType::F64),
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
