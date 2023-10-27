use crate::ast_parsing::{
    get_instruction_effect, get_instruction_type, utils::*, DataType, InstructionType,
    MemoryInstructionType, StackEffect,
};
use itertools::Itertools;
use std::io::Write;
use wast::core::{Func, Instruction};

pub struct ModuleTransformer<'a> {
    raw_text: &'a str,
    output_writer: Box<dyn Write>,
    pub current_func: Option<&'a Func<'a>>,
    pub current_stack: Vec<DataType>,
    pub current_func_base_name: String,
    pub current_split_index: u8,
    pub utx_function_names: Vec<String>,
}

impl<'a> ModuleTransformer<'a> {
    pub fn new(raw_text: &'a str, output_writer: Box<dyn Write>) -> Self {
        ModuleTransformer {
            raw_text,
            output_writer,
            current_func: None,
            current_stack: Vec::default(),
            current_func_base_name: String::default(),
            current_split_index: 0,
            utx_function_names: Vec::default(),
        }
    }

    fn writeln(&mut self, text: &str, indent: usize) {
        let formatted_text = format!("{}{text}\n", INDENTATION_STR.repeat(indent));
        self.output_writer
            .write(formatted_text.as_ref())
            .expect("Could not write");
    }

    pub fn emit_function_split(
        &mut self,
        culprit_instruction: MemoryInstructionType,
        next_instructions: &'_ [Instruction],
    ) -> String {
        self.current_split_index += 1;

        let new_func_name = format!(
            "{}_{}",
            self.current_func_base_name, self.current_split_index
        );

        let new_func_signature = format!("(func ${new_func_name} {TRANSACTION_FUNCTION_SIGNATURE}");

        match culprit_instruction {
            MemoryInstructionType::Load { ty, offset } => {
                self.emit_load_split(&new_func_signature, next_instructions, ty, offset);
            }
            MemoryInstructionType::Store { ty, offset } => {
                self.emit_store_split(&new_func_signature, next_instructions, ty, offset);
            }
        }

        new_func_name
    }

    fn emit_split(
        &mut self,
        new_func_signature: &str,
        naddr: usize,
        next_instructions: &'_ [Instruction],
        pre_split: &[&str],
        post_split: &[&str],
    ) {
        for &pre_split_instr in pre_split {
            self.writeln(pre_split_instr, INSTRUCTION_INDENT);
        }
        self.writeln("local.get $utx", INSTRUCTION_INDENT);
        self.writeln(&format!("i32.const {naddr}"), INSTRUCTION_INDENT);
        self.writeln("i32.store8 offset=63", INSTRUCTION_INDENT);
        self.emit_save_stack();
        self.writeln(
            &format!("i32.const {}", self.utx_function_names.len() + 1),
            INSTRUCTION_INDENT,
        );
        self.emit_end_expression(MODULE_MEMBER_INDENT);
        self.writeln(&new_func_signature, MODULE_MEMBER_INDENT);
        self.emit_locals_if_necessary(next_instructions);
        self.emit_restore_stack();
        for &post_split_inst in post_split {
            self.writeln(post_split_inst, INSTRUCTION_INDENT);
        }
    }

    fn emit_load_split(
        &mut self,
        new_func_signature: &str,
        next_instructions: &'_ [Instruction],
        data_type: DataType,
        mem_offset: u64,
    ) {
        self.current_stack.pop();
        let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
        let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
        let offset_const = format!("i32.const {mem_offset}");
        let load_data_type = format!("{}.load", data_type.as_str());
        let pre_split = [
            set_address.as_str(),
            "local.get $utx",
            get_address.as_str(),
            offset_const.as_str(),
            "i32.add",
            "i32.store",
        ];

        let post_split = ["local.get $utx", "i32.load", load_data_type.as_str()];

        self.emit_split(
            new_func_signature,
            1,
            next_instructions,
            &pre_split,
            &post_split,
        );
    }

    fn emit_store_split(
        &mut self,
        new_func_signature: &str,
        next_instructions: &'_ [Instruction],
        data_type: DataType,
        mem_offset: u64,
    ) {
        // current stack state should be (rightmost is top) [.., address, value]
        self.current_stack.pop();
        self.current_stack.pop();
        // Convert these to macros?
        let set_value = format!("local.set ${VALUE_LOCAL_NAME}");
        let get_value = format!("local.get ${VALUE_LOCAL_NAME}");
        let set_address = format!("local.set ${ADDRESS_LOCAL_NAME}");
        let get_address = format!("local.get ${ADDRESS_LOCAL_NAME}");
        let store_data_type = format!("{}.store", data_type.as_str());
        let load_data_type = format!("{}.load", data_type.as_str());
        let offset_const = format!("i32.const {mem_offset}");
        let pre_split = [
            set_value.as_str(),
            set_address.as_str(),
            "local.get $state",
            get_value.as_str(),
            store_data_type.as_str(),
            "local.get $utx",
            get_address.as_str(),
            offset_const.as_str(),
            "i32.add",
            "i32.store",
        ];

        let post_split = [
            "local.get $utx",
            "i32.load",
            "local.get $state",
            load_data_type.as_str(),
            store_data_type.as_str(),
        ];

        self.emit_split(
            new_func_signature,
            1,
            next_instructions,
            &pre_split,
            &post_split,
        );
    }

    pub fn emit_instruction_from_current_function(&mut self, instruction_number: usize) {
        let instruction_str = self
            .current_func
            .and_then(|func| {
                self.raw_text[func.span.offset()..]
                    .split("\n")
                    .nth(instruction_number + 1)
            })
            .unwrap_or("")
            .trim();

        self.writeln(instruction_str, INSTRUCTION_INDENT);
    }

    /// Check the instruction-stream to see if any locals will be needed for stack-juggling.
    /// If so, emit them.
    pub fn emit_locals_if_necessary(&mut self, instructions: &'_ [Instruction]) {
        let mut stack = Vec::<DataType>::new();
        for instruction in instructions {
            if let InstructionType::Memory(ty) = get_instruction_type(instruction) {
                match ty {
                    MemoryInstructionType::Load { .. } => {
                        stack.pop();
                        self.writeln(
                            &format!("(local ${ADDRESS_LOCAL_NAME} i32)"),
                            INSTRUCTION_INDENT,
                        );
                    }
                    MemoryInstructionType::Store { ty, .. } => {
                        stack.pop();
                        stack.pop();
                        self.writeln(
                            &format!("(local ${ADDRESS_LOCAL_NAME} i32)"),
                            INSTRUCTION_INDENT,
                        );
                        self.writeln(
                            &format!("(local ${VALUE_LOCAL_NAME} {})", ty.as_str()),
                            INSTRUCTION_INDENT,
                        );
                    }
                }
                stack
                    .into_iter()
                    .unique()
                    .for_each(|data_type| match data_type {
                        DataType::I32 => self.writeln(
                            &format!("(local $i32_{STACK_SAVE_NAME} i32)"),
                            INSTRUCTION_INDENT,
                        ),
                        DataType::I64 => self.writeln(
                            &format!("(local $i64{STACK_SAVE_NAME} i64)"),
                            INSTRUCTION_INDENT,
                        ),
                        DataType::F32 => self.writeln(
                            &format!("(local $f32{STACK_SAVE_NAME} f32)"),
                            INSTRUCTION_INDENT,
                        ),
                        DataType::F64 => self.writeln(
                            &format!("(local $f64{STACK_SAVE_NAME} f64)"),
                            INSTRUCTION_INDENT,
                        ),
                    });
                break;
            }
            match get_instruction_effect(instruction) {
                StackEffect::Unary(ty) => {
                    stack.pop();
                    stack.push(ty);
                }
                StackEffect::Binary(ty) => {
                    stack.pop();
                    stack.pop();
                    stack.push(ty);
                }
                StackEffect::Add(ty) => stack.push(ty),
                StackEffect::Remove => {
                    stack.pop();
                }
                StackEffect::RemoveTwo => {
                    stack.pop();
                    stack.pop();
                }
            }
        }
    }

    pub fn emit_funcref_table(&mut self) {
        if self.utx_function_names.len() > 0 {
            self.writeln(
                &format!("(table {} funcref)", self.utx_function_names.len() + 1,),
                MODULE_MEMBER_INDENT,
            );

            let joined_names = self
                .utx_function_names
                .iter()
                .map(|name| format!("${name}"))
                .collect::<Vec<String>>()
                .join(" ");

            self.writeln(
                &format!("(elem (i32.const 1) func {})", &joined_names),
                MODULE_MEMBER_INDENT,
            );
        }
    }

    fn emit_save_stack(&mut self) {
        let mut offset = 0;

        let mut instructions = Vec::new();
        for data in self.current_stack.iter().rev() {
            instructions.push(format!("local.set ${}_{STACK_SAVE_NAME}", data.as_str()));
            instructions.push(format!("local.get $state"));
            instructions.push(format!("local.get ${}_{STACK_SAVE_NAME}", data.as_str()));
            instructions.push(format!("{}.store offset={offset}", data.as_str()));
            offset += data.size();
        }

        for instruction in instructions {
            self.writeln(&instruction, INSTRUCTION_INDENT);
        }
    }

    fn emit_restore_stack(&mut self) {
        let mut offset: usize = self.current_stack.iter().map(DataType::size).sum();

        let mut instructions = Vec::new();
        for data in self.current_stack.iter() {
            offset -= data.size();
            instructions.push(format!("local.get $state"));
            instructions.push(format!("{}.load offset={offset}", data.as_str()));
        }

        for instruction in instructions {
            self.writeln(&instruction, INSTRUCTION_INDENT);
        }
    }

    pub fn emit_end_expression(&mut self, indent: usize) {
        self.writeln(")", indent);
    }

    pub fn emit_current_func_signature(&mut self) {
        self.writeln(
            &format!(
                "(func ${} {TRANSACTION_FUNCTION_SIGNATURE}",
                self.current_func_base_name
            ),
            MODULE_MEMBER_INDENT,
        );
    }

    pub fn emit_memory(&mut self) {
        self.writeln("(memory 10)", MODULE_MEMBER_INDENT);
    }

    pub fn emit_module_start(&mut self) {
        self.writeln("(module", MODULE_INDENT);
    }

    /// Emits all the text from the given offset until the closing parenthesis of the section.
    pub fn emit_section(&mut self, from: usize, indent: usize) {
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
        } else {
            panic!("Malformed file, unbalanced parenthesis");
        }
    }

    pub fn emit_utx_type(&mut self) {
        self.writeln(
            "(type $utx_f (func (param i32 i32 i32) (result i32)))",
            MODULE_MEMBER_INDENT,
        );
    }
}
