use std::io::Write;

use rand::{distributions::Alphanumeric, Rng};
use wast::core::{Export, Func, Instruction, Type};
use wast::Wat;

use crate::ast_parsing::ast::{walk_ast, AstWalker};
use crate::ast_parsing::instruction_analysis::{
    get_instruction_type, DataType, InstructionType, MemoryInstructionType,
};

// This result might have to be variable
const TRANSACTION_FUNCTION_SIGNATURE: &str =
    "(type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)";
const IGNORE_FUNC_PREFIX: &str = "__";
const ADDRESS_LOCAL_NAME: &str = "memory_address";
const VALUE_LOCAL_NAME: &str = "value_to_store";
const INSTRUCTION_INDENT: usize = 2;
const MODULE_MEMBER_INDENT: usize = 1;
const MODULE_INDENT: usize = 0;
const INDENTATION_STR: &str = "    ";

pub fn transform_emit_ast(ast: &Wat, raw_text: &str, writer: Box<dyn Write>) {
    walk_ast(ast, Box::new(ModuleTransformer::new(raw_text, writer)))
}

struct ModuleTransformer<'a> {
    raw_text: &'a str,
    output_writer: Box<dyn Write>,
    current_func: Option<&'a Func<'a>>,
    current_func_base_name: String,
    current_split_index: u8,
    utx_function_names: Vec<String>,
}

impl<'a> ModuleTransformer<'a> {
    fn new(raw_text: &'a str, output_writer: Box<dyn Write>) -> Self {
        ModuleTransformer {
            raw_text,
            output_writer,
            current_func: None,
            current_func_base_name: String::default(),
            current_split_index: 0,
            utx_function_names: Vec::default(),
        }
    }

    fn writeln(&mut self, text: &str, indent: usize) {
        let formatted_text = format!("{}{}\n", INDENTATION_STR.repeat(indent), text);
        self.output_writer
            .write(formatted_text.as_ref())
            .expect("Could not write");
    }

    fn emit_function_split(
        &mut self,
        culprit_instruction: MemoryInstructionType,
        next_instructions: &'_ [Instruction],
    ) -> String {
        self.current_split_index += 1;

        let new_func_name = format!(
            "{}_{}",
            self.current_func_base_name, self.current_split_index
        );

        let new_func_signature =
            format!("(func ${new_func_name} {TRANSACTION_FUNCTION_SIGNATURE}",);

        match culprit_instruction {
            MemoryInstructionType::Load(_) => {
                self.emit_load_split(&new_func_signature, next_instructions);
            }
            MemoryInstructionType::Store(data_type) => {
                self.emit_store_split(&new_func_signature, next_instructions, data_type);
            }
            MemoryInstructionType::OtherMem => {
                unimplemented!(
                    "Encountered an unsupported instruction in function {}",
                    self.current_func_base_name
                )
            }
        }

        new_func_name
    }

    fn emit_load_split(&mut self, new_func_signature: &str, next_instructions: &'_ [Instruction]) {
        // current stack state should be [.., address]
        // --PRE-SPLIT--
        self.writeln(
            &format!("local.set ${ADDRESS_LOCAL_NAME}"),
            INSTRUCTION_INDENT,
        );
        // get the base address of utx->addrs
        self.emit_get_utx();
        self.writeln(
            &format!("local.get ${ADDRESS_LOCAL_NAME}"),
            INSTRUCTION_INDENT,
        );
        // store address for load instruction
        self.writeln("i32.store", INSTRUCTION_INDENT);

        self.emit_get_utx();
        self.writeln("i32.const 1", INSTRUCTION_INDENT);
        self.writeln("i32.store8 offset=63", INSTRUCTION_INDENT);

        // return table index for next function
        let table_index = self.utx_function_names.len();
        self.writeln(&format!("i32.const {table_index}"), INSTRUCTION_INDENT);
        self.emit_end_expression(MODULE_MEMBER_INDENT);

        // --POST-SPLIT--
        self.writeln(&new_func_signature, MODULE_MEMBER_INDENT);
        self.emit_locals_if_necessary(next_instructions);

        // load address from utx->addrs[0]
        self.emit_get_utx();
        self.writeln("i32.load", INSTRUCTION_INDENT);
    }

    fn emit_store_split(
        &mut self,
        new_func_signature: &str,
        next_instructions: &'_ [Instruction],
        data_type: DataType,
    ) {
        let data_type = data_type.as_str();
        // current stack state should be (rightmost is top) [.., address, value]
        // --PRE-SPLIT--
        // operands need to be changed to be in the correct order
        // (value, address) -> (address, value)

        // save value to be stored in $value
        self.writeln(
            &format!("local.set ${VALUE_LOCAL_NAME}"),
            INSTRUCTION_INDENT,
        );
        // save address to store at in $address
        self.writeln(
            &format!("local.set ${ADDRESS_LOCAL_NAME}"),
            INSTRUCTION_INDENT,
        );

        // get the base address of state
        self.emit_get_state();
        self.writeln(
            &format!("local.get ${VALUE_LOCAL_NAME}"),
            INSTRUCTION_INDENT,
        );
        // store value for store instruction
        self.writeln(&format!("{data_type}.store"), INSTRUCTION_INDENT);
        // get the base address of utx->addrs
        self.emit_get_utx();
        self.writeln(
            &format!("local.get ${ADDRESS_LOCAL_NAME}"),
            INSTRUCTION_INDENT,
        );
        // store address for load instruction
        self.writeln("i32.store", INSTRUCTION_INDENT);

        self.emit_get_utx();
        self.writeln("i32.const 1", INSTRUCTION_INDENT);
        self.writeln("i32.store8 offset=63", INSTRUCTION_INDENT);

        // return table index for next function
        let table_index = self.utx_function_names.len();
        self.writeln(&format!("i32.const {table_index}"), INSTRUCTION_INDENT);
        self.emit_end_expression(MODULE_MEMBER_INDENT);
        // --POST-SPLIT--
        self.writeln(&new_func_signature, MODULE_MEMBER_INDENT);
        self.emit_locals_if_necessary(next_instructions);

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

    fn emit_instruction_from_current_function(&mut self, instruction_number: usize) {
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
    fn emit_locals_if_necessary(&mut self, instructions: &'_ [Instruction]) {
        on_next_mem_instruction(instructions, |instruction_type| match instruction_type {
            MemoryInstructionType::Load(_) => {
                self.writeln(
                    &format!("(local ${ADDRESS_LOCAL_NAME} i32)"),
                    INSTRUCTION_INDENT,
                );
            }
            MemoryInstructionType::Store(data_type) => {
                self.writeln(
                    &format!("(local ${ADDRESS_LOCAL_NAME} i32)"),
                    INSTRUCTION_INDENT,
                );
                self.writeln(
                    &format!("(local ${VALUE_LOCAL_NAME} {})", data_type.as_str()),
                    INSTRUCTION_INDENT,
                );
            }
            MemoryInstructionType::OtherMem => {}
        });
    }

    fn emit_funcref_table(&mut self) {
        if self.utx_function_names.len() > 0 {
            self.writeln(
                &format!("(table {} funcref)", self.utx_function_names.len()),
                MODULE_MEMBER_INDENT,
            );

            let joined_names = self
                .utx_function_names
                .iter()
                .map(|name| format!("${name}"))
                .collect::<Vec<String>>()
                .join(" ");

            self.writeln(
                &format!("(elem (i32.const 0) {})", &joined_names),
                MODULE_MEMBER_INDENT,
            );
        }
    }

    fn emit_current_func_signature(&mut self) {
        self.writeln(
            &format!(
                "(func ${} {TRANSACTION_FUNCTION_SIGNATURE}",
                self.current_func_base_name
            ),
            MODULE_MEMBER_INDENT,
        );
    }

    fn emit_end_expression(&mut self, indent: usize) {
        self.writeln(")", indent);
    }

    fn emit_memory(&mut self) {
        self.writeln("(memory 10)", MODULE_MEMBER_INDENT);
    }

    fn emit_module_start(&mut self) {
        self.writeln("(module", MODULE_INDENT);
    }

    fn emit_get_utx(&mut self) {
        self.writeln("local.get $utx", INSTRUCTION_INDENT);
    }

    fn emit_get_state(&mut self) {
        self.writeln("local.get $state", INSTRUCTION_INDENT);
    }

    /// Emits all the text from the given offset until the closing parenthesis of the section.
    fn emit_section(&mut self, from: usize, indent: usize) {
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
}

impl<'a> AstWalker<'a> for ModuleTransformer<'a> {
    type WalkResult = ();
    fn setup(&mut self) {
        self.emit_module_start();
    }

    fn handle_func(&mut self, func: &'a Func, instructions: &'a [Instruction]) {
        let new_basename = match func.id.map(|id| id.name()) {
            None => gen_random_func_name(),
            Some(func_name) => {
                if func_name.starts_with(IGNORE_FUNC_PREFIX) {
                    self.emit_section(func.span.offset(), MODULE_MEMBER_INDENT);
                    return;
                }
                func_name.into()
            }
        };

        self.utx_function_names.push(new_basename.clone());
        self.current_func_base_name = new_basename;

        self.current_func = Some(func);
        self.current_split_index = 0;
        self.emit_current_func_signature();
        self.emit_locals_if_necessary(instructions);
        for (i, instruction) in instructions.iter().enumerate() {
            if let InstructionType::Memory(instruction_type) = get_instruction_type(instruction) {
                let split_name =
                    self.emit_function_split(instruction_type, &instructions[(i + 1)..]);
                self.utx_function_names.push(split_name);
            }
            self.emit_instruction_from_current_function(i);
        }
        self.emit_end_expression(MODULE_MEMBER_INDENT);
    }

    fn handle_export(&mut self, export: &'a Export) {
        self.emit_section(export.span.offset(), MODULE_MEMBER_INDENT);
    }

    fn handle_type(&mut self, ty: &'a Type) {
        self.emit_section(ty.span.offset(), MODULE_MEMBER_INDENT);
    }

    fn finish_and_build_result(&mut self) -> Self::WalkResult {
        self.emit_memory();
        self.emit_funcref_table();
        self.emit_end_expression(MODULE_INDENT);
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

fn on_next_mem_instruction<F>(instructions: &'_ [Instruction], mut f: F)
where
    F: FnMut(MemoryInstructionType) -> (),
{
    if let Some(instruction_type) = instructions
        .iter()
        .map(get_instruction_type)
        .find(InstructionType::is_mem_access_instruction)
        .and_then(|instruction_type| match instruction_type {
            InstructionType::Memory(memory_instruction_type) => Some(memory_instruction_type),
            InstructionType::Benign => None,
        })
    {
        f(instruction_type);
    };
}
