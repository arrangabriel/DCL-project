use std::fmt::{Display, Formatter};

use wast::core::Instruction as WastInstruction;
use wast::core::Instruction::{
    Block, Br, BrIf, Drop, End, F32Const, F32Gt, F64Const, F64Gt, I32Add, I32Const, I32Eq, I32Eqz, I32GtS, I32GtU, I32Load, I32Load16u, I32LtS, I32LtU, I32Mul, I32Ne, I32Shl, I32Store, I32Store8, I32Sub, I32WrapI64, I32Xor, I64Add, I64Const, I64Eq, I64ExtendI32U, I64GtS, I64GtU, I64Load, I64Load32u, I64LtS, I64LtU, I64Mul, I64Ne, I64Sub, I64Xor, LocalGet, LocalSet, LocalTee, Return};
use wast::token::Index;

use crate::chop_up::constants::UTX_LOCALS;
use crate::chop_up::instruction::DataType;

pub struct Instruction<'a> {
    pub instr: &'a WastInstruction<'a>,
    pub raw_text: &'a str,
    pub index: usize,
    pub stack: Vec<StackValue>,
    pub scopes: Vec<Scope>,
}

impl<'a> Instruction<'a> {
    pub fn new(
        instr: &'a WastInstruction<'a>,
        raw_text: &'a str,
        index: usize,
        stack: Vec<StackValue>,
        scopes: Vec<Scope>,
    ) -> Self {
        let raw_text = raw_text.trim();
        Instruction {
            instr,
            raw_text,
            index,
            stack,
            scopes,
        }
    }
}

#[derive(Clone)]
pub struct Scope {
    pub ty: ScopeType,
    pub name: Option<String>,
    pub stack_start: usize,
}

#[derive(Clone)]
pub enum ScopeType {
    Block,
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

    pub fn from_wast_instruction(instruction: &WastInstruction, local_types: &[DataType]) -> Self {
        match instruction {
            Return // The return might have to be handled with care
            | End(_) | Block(_) | Br(_) => StackEffect::new(0, None, false, false),
            LocalGet(index) => {
                let (ty, is_safe) = type_and_safety_from_param(index, local_types);
                StackEffect::new(0, Some(ty), is_safe, true)
            }
            LocalTee(_) => StackEffect::new(0, None, false, false),
            I64Load(_) | I64Load32u(_) | I64ExtendI32U => StackEffect::new(1, Some(DataType::I64), false, false),
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
            _ => panic!("Unsupported instruction read when producing StackEffect - {:?}", instruction),
        }
    }

    pub fn from_instruction(instruction: &Instruction, local_types: &[DataType]) -> Self {
        Self::from_wast_instruction(instruction.instr, local_types)
    }
}

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
        // TODO - to be completely safe we need to check the type of id'd locals
        // in case compiled code uses them (usually not)
        Index::Id(id) => (DataType::I32, name_is_param(id.name())),
    }
}

fn name_is_param(name: &str) -> bool {
    match name {
        "tx" | "state" => true,
        _ => false,
    }
}

/// Assuming use in a function of the type (tx, state) -> ?
fn index_is_param(index: usize) -> bool {
    index < 3
}

