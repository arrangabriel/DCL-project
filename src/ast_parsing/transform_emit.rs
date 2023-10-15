use std::io::Write;

use rand::{distributions::Alphanumeric, Rng};
use wast::core::{Func, Instruction, Memory, Module};
use wast::Wat;

use crate::ast_parsing::ast::{walk_ast, AstWalker};
use crate::instruction_analysis::{
    get_instruction_type, DataType, InstructionType, MemoryInstructionType,
};

// This result might have to be variable
const TRANSACTION_FUNCTION_SIGNATURE: &str =
    "(param $tx i32) (param $utx i32) (param $state i32) (result i32)";
const INSTRUCTION_INDENT: usize = 2;
const FUNCTION_INDENT: usize = 1;

pub fn transform_emit_ast(ast: &Wat, raw_text: &str, writer: Box<dyn Write>) {
    walk_ast(ast, Box::new(ModuleTransformer::new(raw_text, writer)))
}

struct ModuleTransformer<'a> {
    raw_text: &'a str,
    output_writer: Box<dyn Write>,
    current_func: Option<&'a Func<'a>>,
    current_func_base_name: String,
    current_split_index: u8,
    split_function_names: Vec<String>,
}

impl<'a> ModuleTransformer<'a> {
    fn new(raw_text: &'a str, output_writer: Box<dyn Write>) -> Self {
        ModuleTransformer {
            raw_text,
            output_writer,
            current_func: None,
            current_func_base_name: String::default(),
            current_split_index: 0,
            split_function_names: Vec::default(),
        }
    }

    fn writeln(&mut self, text: &str, indent: usize) {
        self.write(&format!("{text}\n"), indent);
    }

    fn write(&mut self, text: &str, indent: usize) {
        let formatted_text = format!("{}{}", "  ".repeat(indent), text);
        self.output_writer
            .write(formatted_text.as_ref())
            .expect("Could not write");
    }

    fn emit_section(
        &mut self,
        offset: usize,
        ending_delimiter: &str,
        indent: usize,
        postfix: &str,
    ) {
        let section_text = &self.raw_text[offset - 1..];
        let section_end = section_text[1..]
            .find(ending_delimiter)
            .expect("Could not find next section start");
        let section_text = format!("{}{}", &section_text[..=section_end], postfix);
        self.writeln(&section_text, indent)
    }

    fn emit_function_split(&mut self, culprit_instruction: MemoryInstructionType) {
        self.current_split_index += 1;

        let new_func_name = format!(
            "${}_{}",
            self.current_func_base_name, self.current_split_index
        );

        let new_func_signature = format!("(func {new_func_name} {TRANSACTION_FUNCTION_SIGNATURE}",);

        match culprit_instruction {
            MemoryInstructionType::Load(_) => self.emit_load_split(&new_func_signature),
            MemoryInstructionType::Store(data_type) => {
                self.emit_store_split(&new_func_signature, data_type)
            }
            MemoryInstructionType::OtherMem => {
                unimplemented!(
                    "Encountered an unsupported instruction in function {}",
                    self.current_func_base_name
                )
            }
        }

        self.split_function_names.push(new_func_name);
    }

    fn emit_load_split(&mut self, new_func_signature: &str) {
        // current stack state should be [.., address]
        // --PRE-SPLIT--
        self.writeln("local.set $address", INSTRUCTION_INDENT);
        // get the base address of utx->addrs
        self.emit_get_utx();

        self.writeln("local.get $address", INSTRUCTION_INDENT);
        // store address for load instruction
        self.writeln("i32.store", INSTRUCTION_INDENT);

        // return table index for next function
        let table_index = self.split_function_names.len();
        self.writeln(&format!("i32.const {table_index}"), INSTRUCTION_INDENT);
        self.writeln(")", FUNCTION_INDENT);

        // --POST-SPLIT--
        self.writeln(&new_func_signature, FUNCTION_INDENT);

        // load address from utx->addrs[0]
        self.emit_get_utx();
        self.writeln("i32.load", INSTRUCTION_INDENT);
    }

    fn emit_store_split(&mut self, new_func_signature: &str, data_type: DataType) {
        let data_type = data_type.as_str();
        // current stack state should be (rightmost is top) [.., address, value]
        // --PRE-SPLIT--
        // operands need to be changed to be in the correct order
        // (value, address) -> (address, value)

        // save value to be stored in $value
        self.writeln("local.set $value", INSTRUCTION_INDENT);
        // save address to store at in $address
        self.writeln("local.set $address", INSTRUCTION_INDENT);

        // get the base address of state
        self.emit_get_state();
        self.writeln("local.get $value", INSTRUCTION_INDENT);
        // store value for store instruction
        self.writeln(&format!("{data_type}.store"), INSTRUCTION_INDENT);
        // get the base address of utx->addrs
        self.emit_get_utx();
        self.writeln("local.get $address", INSTRUCTION_INDENT);
        // store address for load instruction
        self.writeln("i32.store", INSTRUCTION_INDENT);

        // return table index for next function
        let table_index = self.split_function_names.len();
        self.writeln(&format!("i32.const {table_index}"), INSTRUCTION_INDENT);
        self.writeln(")", FUNCTION_INDENT);
        // --POST-SPLIT--
        self.writeln(&new_func_signature, FUNCTION_INDENT);
        // load address from utx->addrs[0]
        self.emit_get_utx();
        self.writeln(
            &format!("{}.load", DataType::I32.as_str()),
            INSTRUCTION_INDENT,
        );
        // load value from state[0]
        self.emit_get_state();
        self.writeln(&format!("{data_type}.load"), INSTRUCTION_INDENT);
    }

    fn emit_get_utx(&mut self) {
        self.writeln("local.get $utx", INSTRUCTION_INDENT);
    }

    fn emit_get_state(&mut self) {
        self.writeln("local.get $state", INSTRUCTION_INDENT);
    }

    fn get_instruction_string(&self, instruction_number: usize) -> String {
        let instruction_reference = self
            .current_func
            .and_then(|func| {
                self.raw_text[func.span.offset()..]
                    .split("\n")
                    .nth(instruction_number + 1)
            })
            .unwrap_or("")
            .trim();

        String::from(instruction_reference)
    }
}

impl<'a> AstWalker<'a> for ModuleTransformer<'a> {
    type WalkResult = ();
    fn handle_module(&mut self, module: &Module) {
        self.emit_section(module.span.offset(), "\n", 0, "");
    }

    fn start_handle_func(&mut self, func: &'a Func) {
        let (func_name, postfix) = match func.id {
            Some(id) => (String::from(id.name()), String::default()),
            None => {
                let new_name = gen_random_func_name();
                let postfix = format!(" {}", &new_name);
                (new_name, postfix)
            }
        };

        self.current_func = Some(func);
        self.current_func_base_name = func_name;
        self.current_split_index = 0;
        self.emit_section(func.span.offset(), "\n", 1, &postfix);
    }

    fn handle_func_instructions(&mut self, instructions: &'_ [Instruction]) {
        if let Some(instruction_type) = instructions
            .iter()
            .map(get_instruction_type)
            .find(InstructionType::is_mem_access_instruction)
            .and_then(|instruction_type| match instruction_type {
                InstructionType::Memory(memory_instruction_type) => Some(memory_instruction_type),
                InstructionType::Benign => None,
            })
        {
            match instruction_type {
                MemoryInstructionType::Load(_) => {
                    self.writeln("(local $address i32)", INSTRUCTION_INDENT);
                }
                MemoryInstructionType::Store(data_type) => {
                    self.writeln("(local $address i32)", INSTRUCTION_INDENT);
                    self.writeln(
                        &format!("(local $value {})", data_type.as_str()),
                        INSTRUCTION_INDENT,
                    );
                }
                MemoryInstructionType::OtherMem => {}
            }
        };

        for (i, instruction) in instructions.iter().enumerate() {
            if let InstructionType::Memory(instruction_type) = get_instruction_type(instruction) {
                self.emit_function_split(instruction_type);
            }
            self.writeln(&self.get_instruction_string(i), 2);
        }

        // This might not always be correct
        self.writeln("i32.const 0", INSTRUCTION_INDENT);
        self.writeln(")", 1);
    }

    fn handle_memory(&mut self, memory: &'a Memory) {
        self.emit_section(memory.span.offset(), "\n", 1, "");
    }

    fn finish_and_build_result(&mut self) -> Self::WalkResult {
        if self.split_function_names.len() > 0 {
            self.writeln(
                &format!("(table {} funcref)", self.split_function_names.len()),
                FUNCTION_INDENT,
            );
            let joined_names = self.split_function_names.join(" ");
            self.writeln(
                &format!("(elem (i32.const 0) {})", &joined_names),
                FUNCTION_INDENT,
            );
        }

        self.writeln(")", 0);
        ()
    }
}

fn gen_random_func_name() -> String {
    let rand_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();
    format!("funcid_{rand_id}")
}
