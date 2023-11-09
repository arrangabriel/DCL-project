use std::io::Write;

use crate::ast_parsing::instruction_analysis::{
    needs_split, DataType, InstructionType, MemoryInstructionType, SplitType, StackEffect,
    StackValue,
};
use itertools::Itertools;
use wast::core::{Func, FuncKind, Instruction, ModuleField, ModuleKind};
use wast::Wat;

use crate::ast_parsing::utils::*;

pub fn emit_transformed_wat(
    wat: &Wat,
    raw_text: &str,
    writer: Box<dyn Write>,
    skip_safe_splits: bool,
) -> Result<(), &'static str> {
    let module_fields = match wat {
        Wat::Module(module) => match &module.kind {
            ModuleKind::Text(fields) => Ok(fields),
            ModuleKind::Binary(_) => Err("ModuleKind is binary"),
        },
        Wat::Component(_) => Err("Input module is component"),
    }?;

    let mut transformer = ModuleTransformerV2::new(raw_text, writer, skip_safe_splits);
    transformer.emit_module();
    for field in module_fields {
        match field {
            ModuleField::Func(func) => {
                if let FuncKind::Inline {
                    expression,
                    locals: _,
                } = &func.kind
                {
                    handle_top_level_func(func, &expression.instrs, &mut transformer)?;
                }
            }
            ModuleField::Export(export) => {
                transformer.emit_section(export.span.offset(), MODULE_MEMBER_INDENT)?;
            }
            _ => {
                // Other module fields might need to be handled at a later date
            }
        }
    }
    transformer.emit_end_module();
    Ok(())
}

pub fn handle_top_level_func(
    func: &Func,
    instructions: &[Instruction],
    transformer: &mut ModuleTransformerV2,
) -> Result<(), &'static str> {
    let name = match func.id.map(|id| id.name()) {
        None => gen_random_func_name(),
        Some(func_name) => {
            if func_name.starts_with(IGNORE_FUNC_PREFIX) {
                transformer.emit_section(func.span.offset(), MODULE_MEMBER_INDENT)?;
                return Ok(());
            }
            func_name.into()
        }
    };
    let func_offset = func.span.offset();
    setup_func(&name, instructions, transformer);
    handle_instructions(
        &name,
        func_offset,
        instructions,
        0,
        Vec::default(),
        transformer,
    )
}

fn setup_func(name: &str, instructions: &[Instruction], transformer: &mut ModuleTransformerV2) {
    transformer.emit_utx_func_signature(name);
    transformer.utx_function_names.push(name.into());
    transformer.emit_locals_if_neccessary(instructions);
}

fn handle_instructions(
    name: &str,
    func_offset: usize,
    instructions: &[Instruction],
    instruction_offset: usize,
    mut stack: Vec<StackValue>,
    transformer: &mut ModuleTransformerV2,
) -> Result<(), &'static str> {
    let mut deferred_splits = Vec::new();
    for (i, instruction) in instructions.iter().enumerate() {
        if let Some(split_type) = needs_split(instruction, &stack, transformer.skip_safe_splits)? {
            match split_type {
                SplitType::Normal(culprit_instruction) => {
                    let local_offset = i + 1;
                    let instruction_offset = instruction_offset + local_offset;
                    handle_normal_pre_split(
                        name,
                        func_offset,
                        culprit_instruction,
                        &instructions[local_offset..],
                        instruction_offset,
                        &mut deferred_splits,
                        &mut stack,
                        transformer,
                    );
                    break;
                }
            }
        } else {
            StackEffect::from(instruction).update_stack(&mut stack)?;
            transformer.emit_instruction_from_function(func_offset, instruction_offset + i)?;
        }
    }
    transformer.emit_end_func();
    for deferred_split in deferred_splits {
        handle_defered(deferred_split, transformer)?
    }
    Ok(())
}

fn handle_defered(
    mut deferred_split: DeferredSplit,
    transformer: &mut ModuleTransformerV2,
) -> Result<(), &'static str> {
    setup_func(
        &deferred_split.name,
        deferred_split.instructions,
        transformer,
    );
    transformer.emit_restore_stack(&deferred_split.stack);
    handle_post_split(&mut deferred_split, transformer);
    handle_instructions(
        &deferred_split.name,
        deferred_split.func_offset,
        deferred_split.instructions,
        deferred_split.instruction_offset,
        deferred_split.stack,
        transformer,
    )
}

fn handle_normal_pre_split<'a>(
    base_name: &str,
    func_offset: usize,
    culprit_instruction: MemoryInstructionType,
    instructions: &'a [Instruction],
    instruction_offset: usize,
    deferred_splits: &mut Vec<DeferredSplit<'a>>,
    stack: &mut Vec<StackValue>,
    transformer: &mut ModuleTransformerV2,
) {
    let split_name = format!(
        "{base_name}_{split_index}",
        split_index = deferred_splits.len() + 1
    );
    // perform pre-split
    let pre_split = match culprit_instruction {
        MemoryInstructionType::Load { offset, .. } => {
            stack.pop();
            let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
            let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
            let offset_const = format!("i32.const {offset}");
            vec![
                set_address,
                "local.get $utx".into(),
                get_address,
                offset_const,
                "i32.add".into(),
                "i32.store".into(),
            ]
        }
        MemoryInstructionType::Store { ty, offset } => {
            stack.pop();
            stack.pop();
            let ty = ty.as_str();
            let stack_juggler_local_name = format!("{ty}_{STACK_JUGGLER_NAME}");
            let set_value = format!("local.set ${stack_juggler_local_name}");
            let get_value = format!("local.get ${stack_juggler_local_name}");
            let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
            let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
            let store_data_type = format!("{ty}.store");
            let offset_const = format!("i32.const {offset}");
            vec![
                set_value,
                set_address,
                "local.get $state".into(),
                get_value,
                store_data_type,
                "local.get $utx".into(),
                get_address,
                offset_const,
                "i32.add".into(),
                "i32.store".into(),
            ]
        }
    };
    deferred_splits.push(DeferredSplit {
        name: split_name.clone(),
        func_offset,
        culprit_instruction,
        instructions,
        instruction_offset,
        stack: stack.to_vec(),
    });

    for pre_split_instr in &pre_split {
        transformer.emit_instruction(pre_split_instr, None);
    }
    transformer.emit_instruction("local.get $utx".into(), Some("Save naddr = 1".into()));
    transformer.emit_instruction(&format!("i32.const 1"), None);
    transformer.emit_instruction("i32.store8 offset=63".into(), None);
    transformer.emit_save_stack(&stack);
    transformer.emit_instruction(
        &format!(
            "i32.const {func_index}",
            func_index = transformer.utx_function_names.len()
        ),
        Some("Return index to next microtransaction".into()),
    )
}

fn handle_post_split(deferred_split: &mut DeferredSplit, transformer: &mut ModuleTransformerV2) {
    let post_split = match deferred_split.culprit_instruction {
        MemoryInstructionType::Load { ty, .. } => {
            deferred_split.stack.push(StackValue { ty, is_safe: false });
            let load_data_type = format!("{}.load", ty.as_str());
            vec!["local.get $utx".into(), "i32.load".into(), load_data_type]
        }
        MemoryInstructionType::Store { ty, .. } => {
            let store_data_type = format!("{}.store", ty.as_str());
            let load_data_type = format!("{}.load", ty.as_str());
            vec![
                "local.get $utx".into(),
                "i32.load".into(),
                "local.get $state".into(),
                load_data_type,
                store_data_type,
            ]
        }
    };

    for post_split_instr in &post_split {
        transformer.emit_instruction(post_split_instr, None);
    }
}

impl<'a> ModuleTransformerV2<'a> {
    fn new(raw_text: &'a str, output_writer: Box<dyn Write>, skip_safe_splits: bool) -> Self {
        Self {
            raw_text,
            output_writer,
            skip_safe_splits,
            utx_function_names: Vec::default(),
        }
    }

    fn writeln(&mut self, text: &str, indent: usize) {
        let formatted_text = format!("{}{text}\n", INDENTATION_STR.repeat(indent));
        self.output_writer
            .write(formatted_text.as_ref())
            .expect("Could not write");
    }

    pub fn emit_section(&mut self, from: usize, indent: usize) -> Result<(), &'static str> {
        let mut paren_count = 0;
        let mut to = None;
        let text_to_search = &self.raw_text[from - 1..];
        for (i, c) in text_to_search.chars().enumerate() {
            paren_count += match c {
                '(' => 1,
                ')' => -1,
                _ => 0,
            };
            if paren_count == 0 {
                to = Some(i);
                break;
            }
        }
        if let Some(i) = to {
            self.writeln(&text_to_search[..=i], indent);
            Ok(())
        } else {
            Err("Malformed file, unbalanced parenthesis")
        }
    }

    pub fn emit_locals_if_neccessary(&mut self, instructions: &[Instruction]) {
        if self.skip_safe_splits {
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
            let effect = StackEffect::from(instruction);
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

    pub fn emit_instruction_from_function(
        &mut self,
        func_offset: usize,
        instruction_index: usize,
    ) -> Result<(), &'static str> {
        let instruction_str = self.raw_text[func_offset..]
            // Instructions being separated by a newline might be a scary assumption
            .split("\n")
            .nth(instruction_index + 1)
            .ok_or("Out of bounds access when trying to emit instruction from function")?
            .trim();

        self.writeln(instruction_str, INSTRUCTION_INDENT);
        Ok(())
    }

    pub fn emit_instruction(&mut self, instruction: &str, annotation: Option<String>) {
        let instruction = match annotation {
            Some(annotation) => format!("{instruction:<30};;{annotation}"),
            None => instruction.into(),
        };
        self.writeln(&instruction, INSTRUCTION_INDENT);
    }

    pub fn emit_save_stack(&mut self, stack: &[StackValue]) {
        let mut offset = STATE_BASE_OFFSET;
        let instructions = stack.iter().rev().flat_map(|StackValue { ty, .. }| {
            let ty_str = ty.as_str();
            offset += ty.size();
            [
                format!("local.set ${ty_str}_{STACK_JUGGLER_NAME}"),
                format!("local.get $state"),
                format!("local.get ${ty_str}_{STACK_JUGGLER_NAME}"),
                format!("{ty_str}.store offset={}", offset - ty.size()),
            ]
        });

        for (i, instruction) in instructions.enumerate() {
            let annotation = if i == 0 {
                Some(format!("Save stack - {stack:?}"))
            } else {
                None
            };
            self.emit_instruction(&instruction, annotation);
        }
    }

    pub fn emit_restore_stack(&mut self, stack: &[StackValue]) {
        let mut offset: usize = stack.iter().map(|StackValue { ty, .. }| ty.size()).sum();
        let instructions = stack.iter().flat_map(|StackValue { ty, .. }| {
            offset -= ty.size();
            [
                format!("local.get $state"),
                format!("{}.load offset={offset}", ty.as_str()),
            ]
        });

        for (i, instruction) in instructions.enumerate() {
            let annotation = if i == 0 {
                Some(format!("Restore stack - {stack:?}"))
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
                .map(|name| format!("${name}"))
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
}

pub struct ModuleTransformerV2<'a> {
    raw_text: &'a str,
    output_writer: Box<dyn Write>,
    skip_safe_splits: bool,
    utx_function_names: Vec<String>,
}

struct DeferredSplit<'a> {
    name: String,
    func_offset: usize,
    culprit_instruction: MemoryInstructionType,
    instructions: &'a [Instruction<'a>],
    instruction_offset: usize,
    stack: Vec<StackValue>,
}
