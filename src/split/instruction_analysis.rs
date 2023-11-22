use std::fmt::{Display, Formatter};

use wast::core::Instruction::{I32Sub, LocalSet};
use wast::core::{Instruction, ValType};
use wast::token::Index;
use Instruction::{
    Block, Br, BrIf, DataDrop, Drop, ElemDrop, End, F32Const, F32Gt, F32Load, F32Store, F64Const,
    F64Gt, F64Load, F64Store, GlobalGet, GlobalSet, I32Add, I32Const, I32Eq, I32Eqz, I32GtS,
    I32GtU, I32Load, I32Load16u, I32LtS, I32LtU, I32Mul, I32Ne, I32Shl, I32Store, I32Store8,
    I32WrapI64, I32Xor, I64Add, I64Const, I64Eq, I64ExtendI32U, I64GtS, I64GtU, I64Load, I64LtS,
    I64LtU, I64Mul, I64Ne, I64Store, I64Store8, I64Sub, I64Xor, LocalGet, LocalTee, MemoryCopy,
    MemoryDiscard, MemoryFill, MemoryGrow, MemoryInit, MemorySize, Return, TableCopy, TableFill,
    TableGet, TableGrow, TableInit, TableSet, TableSize,
};

use DataType::*;
use InstructionType::{Benign, Memory};

use crate::split::instruction_analysis::LocalType::{Get, Set, Tee};
use crate::split::transform::Scope;
use crate::split::utils::{index_is_param, name_is_param};

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
        match instruction {
            Return // The return might have to be handled with care
            | End(_) | Block(_) | Br(_) => StackEffect::new(0, None, false, false),
            LocalGet(index) => {
                let (ty, is_safe) = type_and_safety_from_param(index, local_types);
                StackEffect::new(0, Some(ty), is_safe, true)
            }
            LocalTee(_) => StackEffect::new(0, None, false, false),
            I64Load(_) | I64ExtendI32U => StackEffect::new(1, Some(I64), false, false),
            I64Const(_) => StackEffect::new(0, Some(I64), false, false),
            I32WrapI64 | I32Load(_) | I32Load16u(_) | I32Eqz => StackEffect::new(1, Some(I32), false, true),
            I32Const(_) => StackEffect::new(0, Some(I32), false, false),
            I32Mul | I32Add | I32Sub | I32Eq | F64Gt | F32Gt |
            I32GtU | I32GtS | I64GtU | I64GtS | I32LtU |
            I32LtS | I64LtU | I64LtS | I64Eq | I32Ne | I64Ne |
            I32Shl | I32Xor => StackEffect::new(2, Some(I32), false, false),
            I64Mul | I64Add | I64Xor | I64Sub => StackEffect::new(2, Some(I64), false, false),
            I32Store(_) | I32Store8(_) => StackEffect::new(2, None, false, false),
            Drop | BrIf(_) | LocalSet(_) => StackEffect::new(1, None, false, false),
            F64Const(_) => StackEffect::new(0, Some(F64), false, false),
            F32Const(_) => StackEffect::new(0, Some(F32), false, false),
            _ => panic!("Unsupported instruction read when producing StackEffect - {instruction:?}"),
        }
    }
}

const UTX_LOCALS: [DataType; 3] = [I32, I32, I32];

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
        Index::Id(id) => (I32, name_is_param(id.name())),
    }
}

#[derive(PartialEq, Clone)]
pub enum InstructionType {
    Memory(MemoryInstructionType),
    Benign(BenignInstructionType),
}

#[derive(PartialEq, Clone)]
pub enum BenignInstructionType {
    Block(BlockInstructionType),
    IndexedLocal(LocalType, usize),
    Return,
    Other,
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

pub enum SplitType {
    Normal,
    Block,
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
            Benign(match value {
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

fn type_from_load(instruction: &Instruction) -> Option<(DataType, u64)> {
    match instruction {
        I32Load(arg) | I32Load16u(arg) => Some((I32, arg.offset)),
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

/// To be used at some point inside of a scope
pub fn index_of_scope_end(
    instructions_with_index: &[(&Instruction, usize)],
) -> Result<usize, &'static str> {
    let mut scope_level = 1;
    for (i, &(instruction, _)) in instructions_with_index.iter().enumerate() {
        if let Benign(BenignInstructionType::Block(block_instruction_type)) =
            InstructionType::from(instruction)
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
