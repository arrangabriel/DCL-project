use crate::chop_up::instruction::DataType::*;
use crate::chop_up::instruction::InstructionType::{Benign, Memory};
use crate::chop_up::instruction_stream::Instruction;
use crate::chop_up::instruction_stream::StackValue;
use anyhow::{anyhow, Result};
use wast::core::{Instruction as WastInstruction, ValType};
use wast::core::Instruction::{I32Store16, I64Load32u, I64Store16};
use WastInstruction::{
    Block, DataDrop, ElemDrop, End, F32Load, F32Store, F64Load, F64Store, GlobalGet, GlobalSet,
    I32Load, I32Load16u, I32Store, I32Store8, I64Load, I64Store, I64Store8, MemoryCopy,
    MemoryDiscard, MemoryFill, MemoryGrow, MemoryInit, MemorySize, Return, TableCopy, TableFill,
    TableGet, TableGrow, TableInit, TableSet, TableSize,
};

#[derive(PartialEq, Clone)]
pub enum InstructionType {
    Memory(MemoryInstructionType),
    Benign(BenignInstructionType),
}

impl From<&WastInstruction<'_>> for InstructionType {
    fn from(value: &WastInstruction<'_>) -> Self {
        if let Some((ty, offset, subtype)) = type_from_load(value) {
            Memory(MemoryInstructionType::Load {
                ty,
                offset,
                subtype,
            })
        } else if let Some((ty, offset, subtype)) = type_from_store(value) {
            Memory(MemoryInstructionType::Store {
                ty,
                offset,
                subtype,
            })
        } else if is_other_memory_instruction(value) {
            panic!(
                "Unsupported instruction read when producing InstructionType - {:?}",
                value
            )
        } else {
            Benign(match value {
                Block(id) => BenignInstructionType::Block(BlockInstructionType::Block(
                    id.label.map(|id| id.name().into()),
                )),
                End(_) => BenignInstructionType::Block(BlockInstructionType::End),
                Return => BenignInstructionType::Return,
                _ => BenignInstructionType::Other,
            })
        }
    }
}

impl From<&Instruction<'_>> for InstructionType {
    fn from(value: &Instruction) -> Self {
        Self::from(value.instr)
    }
}

#[derive(PartialEq, Clone)]
pub enum BenignInstructionType {
    Block(BlockInstructionType),
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
    Load {
        ty: DataType,
        offset: u64,
        subtype: Option<MemoryInstructionSubtype>,
    },
    Store {
        ty: DataType,
        offset: u64,
        subtype: Option<MemoryInstructionSubtype>,
    },
}

#[derive(Clone, Copy, PartialEq)]
pub enum MemoryInstructionSubtype {
    SixteenU,
    ThirtyTwoU,
    Eight,
    Sixteen
}

impl MemoryInstructionSubtype {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryInstructionSubtype::SixteenU => "16_u",
            MemoryInstructionSubtype::Sixteen => "16",
            MemoryInstructionSubtype::Eight => "8",
            MemoryInstructionSubtype::ThirtyTwoU => "32_u",
        }
    }
}

impl MemoryInstructionType {
    pub fn needs_split(&self, stack: &[StackValue], skip_safe_splits: bool) -> Result<bool> {
        let needs_split = match self {
            MemoryInstructionType::Load { .. } => {
                let last_is_safe = stack
                    .last()
                    .ok_or(anyhow!("Load with empty stack - program is malformed"))?
                    .is_safe;
                !(last_is_safe && skip_safe_splits)
            }
            MemoryInstructionType::Store { .. } => true,
        };
        Ok(needs_split)
    }
}

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

fn type_from_load(
    instruction: &WastInstruction,
) -> Option<(DataType, u64, Option<MemoryInstructionSubtype>)> {
    match instruction {
        I32Load(arg) => Some((I32, arg.offset, None)),
        I32Load16u(arg) => Some((I32, arg.offset, Some(MemoryInstructionSubtype::SixteenU))),
        I64Load(arg) => Some((I64, arg.offset, None)),
        I64Load32u(arg) => Some((I64, arg.offset, Some(MemoryInstructionSubtype::ThirtyTwoU))),
        F32Load(arg) => Some((F32, arg.offset, None)),
        F64Load(arg) => Some((F64, arg.offset, None)),
        _ => None,
    }
}

fn type_from_store(
    instruction: &WastInstruction,
) -> Option<(DataType, u64, Option<MemoryInstructionSubtype>)> {
    match instruction {
        I32Store(arg) => Some((I32, arg.offset, None)),
        I32Store8(arg) => Some((I32, arg.offset, Some(MemoryInstructionSubtype::Eight))),
        I32Store16(arg) => Some((I32, arg.offset, Some(MemoryInstructionSubtype::Sixteen))),
        I64Store(arg) => Some((I64, arg.offset, None)),
        I64Store8(arg) => Some((I64, arg.offset, Some(MemoryInstructionSubtype::Eight))),
        I64Store16(arg) => Some((I64, arg.offset, Some(MemoryInstructionSubtype::Sixteen))),
        F32Store(arg) => Some((F32, arg.offset, None)),
        F64Store(arg) => Some((F64, arg.offset, None)),
        _ => None,
    }
}

fn is_other_memory_instruction(instruction: &WastInstruction) -> bool {
    matches!(
        instruction,
        GlobalGet(_)
            | GlobalSet(_)
            | TableGet(_)
            | TableSet(_)
            | MemorySize(_)
            | MemoryGrow(_)
            | MemoryInit(_)
            | MemoryCopy(_)
            | MemoryFill(_)
            | MemoryDiscard(_)
            | DataDrop(_)
            | ElemDrop(_)
            | TableInit(_)
            | TableCopy(_)
            | TableFill(_)
            | TableSize(_)
            | TableGrow(_)
    )
}
