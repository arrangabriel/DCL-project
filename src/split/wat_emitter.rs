use std::io::Write;

use itertools::Itertools;

use crate::split::function_analysis::{Function, StackEffect, StackValue};
use crate::split::instruction_types::{
    DataType, Instruction, InstructionType, MemoryInstructionType,
};
use crate::split::utils::*;

pub struct WatEmitter {
    output_writer: Box<dyn Write>,
    pub skip_safe_splits: bool,
    pub state_base: usize,
    pub stack_base: usize,
    pub utx_function_names: Vec<(usize, String)>,
    pub current_scope_level: usize,
}

impl WatEmitter {
    pub fn new(output_writer: Box<dyn Write>, state_base: usize, skip_safe_splits: bool) -> Self {
        Self {
            output_writer,
            skip_safe_splits,
            state_base,
            stack_base: state_base + 8,
            utx_function_names: Vec::default(),
            current_scope_level: 0,
        }
    }

    pub fn writeln(&mut self, text: &str, indent: usize) {
        let formatted_text = format!("{}{text}\n", INDENTATION_STR.repeat(indent));
        self.output_writer
            .write(formatted_text.as_ref())
            .expect("Could not write");
    }

    fn emit_existing_locals(&mut self, local_types: &[DataType]) {
        let local_types_str = local_types.iter().map(|ty| ty.as_str()).join(" ");
        if !local_types_str.is_empty() {
            self.emit_instruction(&format!("(local {local_types_str})"), None)
        }
    }

    pub fn emit_locals(&mut self, instructions: &[Instruction], locals: &[DataType]) {
        self.emit_existing_locals(locals);

        // TODO - optimally emit locals
        if true {
            //self.skip_safe_splits {
            self.emit_all_locals();
            return;
        }
        let mut stack = Vec::<DataType>::new();
        for instruction in instructions {
            if let InstructionType::Memory(instr_type) = InstructionType::from(instruction) {
                stack.pop();
                self.writeln(
                    &format!("(local ${ADDRESS_LOCAL_NAME} i32)"),
                    INSTRUCTION_INDENT,
                );
                let mut types: Vec<DataType> = Vec::new();
                if let MemoryInstructionType::Store { ty, .. } = instr_type {
                    stack.pop();
                    types.push(ty);
                }
                types.append(&mut stack);
                types.into_iter().unique().for_each(|data_type| {
                    self.writeln(
                        &format!(
                            "(local ${}_{STACK_JUGGLER_NAME} {})",
                            data_type.as_str(),
                            data_type.as_str()
                        ),
                        INSTRUCTION_INDENT,
                    )
                });
                break;
            }
            let effect = StackEffect::from_instruction(instruction, locals);
            for _ in 0..effect.remove_n {
                stack.pop();
            }
            if let Some(stack_value) = effect.add {
                stack.push(stack_value.ty);
            }
        }
    }

    pub fn emit_all_locals(&mut self) {
        self.writeln(
            &format!("(local ${ADDRESS_LOCAL_NAME} i32)"),
            INSTRUCTION_INDENT,
        );
        let types = [DataType::I32, DataType::I64, DataType::F32, DataType::F64];
        for ty in types {
            self.writeln(
                &format!("(local ${ty}_{STACK_JUGGLER_NAME} {ty})", ty = ty.as_str(),),
                INSTRUCTION_INDENT,
            );
        }
    }

    pub fn emit_utx_func_signature(&mut self, func_name: &str) {
        self.writeln(
            &format!("(func ${} {TRANSACTION_FUNCTION_SIGNATURE}", func_name),
            MODULE_MEMBER_INDENT,
        )
    }

    pub fn emit_instruction(&mut self, instruction: &str, annotation: Option<String>) {
        let instruction = match annotation {
            Some(annotation) => format!("{instruction:<30};;{annotation}"),
            None => instruction.into(),
        };
        self.writeln(&instruction, INSTRUCTION_INDENT + self.current_scope_level);
    }

    pub fn emit_save_stack_and_locals(
        &mut self,
        stack_base: usize,
        stack: &[StackValue],
        from: usize,
        keep_stack: bool,
        locals: &[DataType],
    ) {
        let already_saved_size: usize = stack[..from].iter().map(|value| value.ty.size()).sum();
        let mut offset = stack_base + already_saved_size;
        let stack = &stack[from..];
        let set_flavour = if keep_stack { "tee" } else { "set" };
        let stack_save_instructions = stack.iter().rev().flat_map(|StackValue { ty, .. }| {
            let ty_str = ty.as_str();
            let instructions = [
                format!("local.{set_flavour} ${ty_str}_{STACK_JUGGLER_NAME}"),
                format!("local.get $state"),
                format!("local.get ${ty_str}_{STACK_JUGGLER_NAME}"),
                format!("{ty_str}.store offset={offset}"),
            ];
            offset += ty.size();
            instructions
        });

        for (i, instruction) in stack_save_instructions.enumerate() {
            let annotation = match i {
                0 => Some(format!(
                    "Save stack - [{stack}]",
                    stack = stack.iter().map(|value| value.to_string()).join(", ")
                )),
                3 => Some(format!(
                    "First {n} bytes reserved for user defined state struct and potential store value",
                    n = stack_base
                )),
                _ => None,
            };
            self.emit_instruction(&instruction, annotation);
        }

        let local_save_instructions = locals.iter().enumerate().flat_map(|(i, ty)| {
            let ty_str = ty.as_str();
            let instructions = [
                format!("local.get $state"),
                format!(
                    "local.get {local_index}",
                    local_index = i + UTX_FUNC_PARAM_COUNT
                ),
                format!("{ty_str}.store offset={offset}"),
            ];
            offset += ty.size();
            instructions
        });

        for (i, instruction) in local_save_instructions.enumerate() {
            let annotation = match i {
                0 => Some(format!(
                    "Save locals - [{locals}]",
                    locals = locals.iter().map(|ty| ty.as_str()).join(", ")
                )),
                _ => None,
            };
            self.emit_instruction(&instruction, annotation);
        }
    }

    pub fn emit_restore_stack(
        &mut self,
        stack_base: usize,
        stack: &[StackValue],
        from: usize,
        until: usize,
    ) {
        let stack_size: usize = stack[..until]
            .iter()
            .map(|StackValue { ty, .. }| ty.size())
            .sum();
        let mut offset = stack_base + stack_size;
        let stack = &stack[from..until];
        let instructions = stack.iter().flat_map(|StackValue { ty, .. }| {
            offset -= ty.size();
            [
                format!("local.get $state"),
                format!("{}.load offset={offset}", ty.as_str()),
            ]
        });

        for (i, instruction) in instructions.enumerate() {
            let annotation = match i {
                0 => Some(format!(
                    "Restore stack - [{stack}]",
                    stack = stack.iter().map(|value| value.to_string()).join(", ")
                )),
                1 => Some(format!(
                    "First {n} bytes reserved for user defined state struct and potential store value",
                    n = stack_base
                )),
                _ => None,
            };
            self.emit_instruction(&instruction, annotation);
        }
    }

    pub fn emit_restore_locals(
        &mut self,
        locals: &[DataType],
        stack_base: usize,
        stack: &[StackValue],
    ) {
        let mut offset = stack_base
            + stack
                .iter()
                .map(|StackValue { ty, .. }| ty.size())
                .sum::<usize>();
        let instructions = locals.iter().enumerate().flat_map(|(i, ty)| {
            let ty_str = ty.as_str();
            let instructions = [
                format!("local.get $state"),
                format!("{ty_str}.load offset={offset}"),
                format!(
                    "local.set {local_index}",
                    local_index = i + UTX_FUNC_PARAM_COUNT
                ),
            ];
            offset += ty.size();
            instructions
        });
        for (i, instruction) in instructions.enumerate() {
            let annotation = if i == 0 {
                Some(format!(
                    "Restore locals [{locals}]",
                    locals = locals.iter().map(|ty| ty.as_str()).join(", ")
                ))
            } else {
                None
            };
            self.emit_instruction(&instruction, annotation);
        }
    }

    pub fn emit_funcref_table(&mut self) {
        if self.utx_function_names.len() > 0 {
            self.writeln(
                &format!("(table {} funcref)", self.utx_function_names.len() + 1,),
                MODULE_MEMBER_INDENT,
            );

            let function_names = self
                .utx_function_names
                .iter()
                .map(|(_, name)| format!("${name}"))
                .collect::<Vec<String>>()
                .join(" ");

            self.writeln(
                &format!("(elem (i32.const 1) func {})", &function_names),
                MODULE_MEMBER_INDENT,
            );
        }
    }

    pub fn emit_module(&mut self) {
        self.writeln("(module", MODULE_INDENT);
    }

    pub fn emit_end_module(&mut self) {
        self.emit_funcref_table();
        self.writeln("(memory 10)", MODULE_MEMBER_INDENT);
        self.writeln(
            "(type $utx_f (func (param i32 i32 i32) (result i32)))",
            MODULE_MEMBER_INDENT,
        );
        self.writeln(")", MODULE_INDENT);
    }

    pub fn emit_end_func(&mut self) {
        self.writeln(")", MODULE_MEMBER_INDENT);
    }

    pub fn emit_function(&mut self, func: &Function) {
        self.writeln(func.signature, MODULE_MEMBER_INDENT);
        if !&func.instructions.is_empty() {
            self.emit_existing_locals(&func.local_types);
            for instruction in &func.instructions {
                self.emit_instruction(instruction.raw_text, None);
            }
            self.emit_end_func();
        }
    }
}
