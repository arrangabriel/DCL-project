use std::io::Write;

use wast::core::{Func, Instruction, Module};
use wast::Wat;

use crate::ast_parsing::ast::{walk_ast, AstWalker};
use crate::module_analysis::is_mem_access_instruction;

pub fn transform_emit_ast(ast: &Wat, raw_text: &str, writer: Box<dyn Write>) {
    walk_ast(ast, Box::new(ModuleTransformer::new(raw_text, writer)))
}

struct ModuleTransformer<'a> {
    raw_text: &'a str,
    output_writer: Box<dyn Write>,
    current_func: Option<&'a Func<'a>>,
    current_func_base_name: &'a str,
    current_split_index: u8,
}

impl<'a> ModuleTransformer<'a> {
    fn new(raw_text: &'a str, output_writer: Box<dyn Write>) -> Self {
        ModuleTransformer {
            raw_text,
            output_writer,
            current_func: None,
            current_func_base_name: "",
            current_split_index: 0,
        }
    }

    fn write(&mut self, text: &str, indent: usize, postfix: &str) {
        let formatted_text = format!("{}{}{postfix}\n", "  ".repeat(indent), text);
        self.output_writer
            .write(formatted_text.as_ref())
            .expect("Could not write");
    }

    fn write_instruction_from_current_function(&mut self, instruction_number: usize) {
        let raw_instruction = &String::from(self.get_raw_instruction(instruction_number));
        self.write(raw_instruction, 2, "");
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
        self.write(&section_text[..=section_end], indent, postfix)
    }

    fn emit_function_split(&mut self) {
        self.current_split_index += 1;
        let new_func_name = format!(
            "{}_{}",
            self.current_func_base_name, self.current_split_index
        );
        let call_instruction = format!("call ${new_func_name}");
        self.write(&call_instruction, 2, "");
        self.write(")", 1, "");

        let new_func_signature = format!("(func ${new_func_name}");

        self.write(&new_func_signature, 1, "");
    }

    fn get_raw_instruction(&self, instruction_number: usize) -> &str {
        self.current_func
            .and_then(|func| {
                self.raw_text[func.span.offset()..]
                    .split("\n")
                    .nth(instruction_number + 1)
            })
            .unwrap_or("")
            .trim()
    }
}

impl<'a> AstWalker<'a> for ModuleTransformer<'a> {
    type WalkResult = ();
    fn handle_module(&mut self, module: &Module) {
        self.emit_section(module.span.offset(), "\n", 0, "");
    }

    fn start_handle_func(&mut self, func: &'a Func) {
        self.current_func = Some(func);
        self.current_func_base_name = func.id.map(|id| id.name()).unwrap_or("anonymous");

        self.emit_section(func.span.offset(), "\n", 1, "");
    }

    fn handle_func_instructions(&mut self, instructions: &'_ [Instruction]) {
        for (i, instruction) in instructions.iter().enumerate() {
            if is_mem_access_instruction(instruction) {
                self.emit_function_split();
            }
            self.write_instruction_from_current_function(i);
        }
        self.write("", 1, ")");
    }

    fn finish_and_build_result(&mut self) -> Self::WalkResult {
        self.write(")", 0, "");
        ()
    }
}
