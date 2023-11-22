use crate::split::function_analysis::{SplitType, StackValue};
use crate::split::instruction_types::DataType::*;
use crate::split::instruction_types::InstructionType::{Benign, Memory};
use crate::split::instruction_types::LocalType::{Get, Set, Tee};
use crate::split::transform::Scope;
use wast::core::Instruction::LocalSet;
use wast::core::{Instruction as WastInstruction, ValType};
use wast::token::Index;
use WastInstruction::{
    Block, DataDrop, ElemDrop, End, F32Load, F32Store, F64Load, F64Store, GlobalGet, GlobalSet,
    I32Load, I32Load16u, I32Store, I32Store8, I64Load, I64Store, I64Store8, LocalGet, LocalTee,
    MemoryCopy, MemoryDiscard, MemoryFill, MemoryGrow, MemoryInit, MemorySize, Return, TableCopy,
    TableFill, TableGet, TableGrow, TableInit, TableSet, TableSize,
};

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

impl From<ValType<'_>> for DataType {
    fn from(value: ValType) -> Self {
        match value {
            ValType::I32 => I32,
            ValType::I64 => I64,
            ValType::F32 => F32,
            ValType::F64 => F64,
            _ => panic!("Unsupported type {:?}", value),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum InstructionType {
    Memory(MemoryInstructionType),
    Benign(BenignInstructionType),
}

impl From<&Instruction<'_>> for InstructionType {
    fn from(value: &Instruction) -> Self {
        if let Some((ty, offset)) = type_from_load(value) {
            Memory(MemoryInstructionType::Load { ty, offset })
        } else if let Some((ty, offset)) = type_from_store(value) {
            Memory(MemoryInstructionType::Store { ty, offset })
        } else if is_other_memory_instruction(value) {
            panic!(
                "Unsupported instruction read when producing InstructionType - {:?}",
                value.instr
            )
        } else {
            Benign(match value.instr {
                Block(id) => BenignInstructionType::Block(BlockInstructionType::Block(
                    id.label.map(|id| id.name().into()),
                )),
                End(_) => BenignInstructionType::Block(BlockInstructionType::End),
                LocalGet(Index::Num(index, _)) => {
                    BenignInstructionType::IndexedLocal(Get, *index as usize)
                }
                LocalSet(Index::Num(index, _)) => {
                    BenignInstructionType::IndexedLocal(Set, *index as usize)
                }
                LocalTee(Index::Num(index, _)) => {
                    BenignInstructionType::IndexedLocal(Tee, *index as usize)
                }
                Return => BenignInstructionType::Return,
                _ => BenignInstructionType::Other,
            })
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum BenignInstructionType {
    Block(BlockInstructionType),
    IndexedLocal(LocalType, usize),
    Return,
    Other,
}

#[derive(PartialEq, Clone)]
pub enum BlockInstructionType {
    End,
    Block(Option<String>),
}

#[derive(Clone, Copy, PartialEq)]
pub enum MemoryInstructionType {
    Load { ty: DataType, offset: u64 },
    Store { ty: DataType, offset: u64 },
}

impl MemoryInstructionType {
    pub fn needs_split(
        &self,
        stack: &[StackValue],
        scopes: &[Scope],
        skip_safe_splits: bool,
    ) -> Result<Option<SplitType>, &'static str> {
        let needs_split = match self {
            MemoryInstructionType::Load { .. } => {
                let last_is_safe = stack
                    .last()
                    .ok_or("Load with empty stack - program is malformed")?
                    .is_safe;
                if last_is_safe && skip_safe_splits {
                    false
                } else {
                    true
                }
            }
            MemoryInstructionType::Store { .. } => true,
        };
        if !needs_split {
            return Ok(None);
        }
        let split_type = if scopes.is_empty() {
            SplitType::Normal
        } else {
            SplitType::Block
        };
        Ok(Some(split_type))
    }
}

#[derive(PartialEq, Clone)]
pub enum LocalType {
    Get,
    Set,
    Tee,
}

impl LocalType {
    pub fn as_str(&self) -> &str {
        match self {
            Get => "get",
            Set => "set",
            Tee => "tee",
        }
    }
}

pub struct Instruction<'a> {
    pub instr: &'a WastInstruction<'a>,
    pub raw_text: &'a str,
    pub index: usize,
}

impl<'a> Instruction<'a> {
    pub fn new(instr: &'a WastInstruction<'a>, raw_text: &'a str, index: usize) -> Self {
        let raw_text = raw_text.trim();
        Instruction {
            instr,
            raw_text,
            index,
        }
    }
}

fn type_from_load(instruction: &Instruction) -> Option<(DataType, u64)> {
    match instruction.instr {
        I32Load(arg) | I32Load16u(arg) => Some((I32, arg.offset)),
        I64Load(arg) => Some((I64, arg.offset)),
        F32Load(arg) => Some((F32, arg.offset)),
        F64Load(arg) => Some((F64, arg.offset)),
        _ => None,
    }
}

fn type_from_store(instruction: &Instruction) -> Option<(DataType, u64)> {
    match instruction.instr {
        I32Store(arg) | I32Store8(arg) => Some((I32, arg.offset)),
        I64Store(arg) | I64Store8(arg) => Some((I64, arg.offset)),
        F32Store(arg) => Some((F32, arg.offset)),
        F64Store(arg) => Some((F64, arg.offset)),
        _ => None,
    }
}

fn is_other_memory_instruction(instruction: &Instruction) -> bool {
    match instruction.instr {
        GlobalGet(_) | GlobalSet(_) | TableGet(_) | TableSet(_) | MemorySize(_) | MemoryGrow(_)
        | MemoryInit(_) | MemoryCopy(_) | MemoryFill(_) | MemoryDiscard(_) | DataDrop(_)
        | ElemDrop(_) | TableInit(_) | TableCopy(_) | TableFill(_) | TableSize(_)
        | TableGrow(_) => true,
        _ => false,
    }
}
