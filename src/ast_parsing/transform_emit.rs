use std::io::Write;

use wast::core::{Export, Func, Instruction, Type};
use wast::Wat;

use crate::ast_parsing::ast::{walk_ast, AstWalker};
use crate::ast_parsing::instruction_analysis::{get_instruction_type, InstructionType};
use crate::ast_parsing::module_transformer::ModuleTransformer;
use crate::ast_parsing::utils::{
    gen_random_func_name, IGNORE_FUNC_PREFIX, MODULE_INDENT, MODULE_MEMBER_INDENT,
};
use crate::ast_parsing::{get_instruction_effect, StackEffect};

pub fn transform_emit_ast(ast: &Wat, raw_text: &str, writer: Box<dyn Write>) {
    walk_ast(ast, Box::new(ModuleTransformer::new(raw_text, writer)))
}

impl<'a> AstWalker<'a> for ModuleTransformer<'a> {
    type WalkResult = ();
    fn setup(&mut self) {
        self.emit_module_start();
        self.emit_utx_type();
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
                let split_name = self.emit_function_split(instruction_type, &instructions[i + 1..]);
                self.utx_function_names.push(split_name);
            } else {
                match get_instruction_effect(instruction) {
                    StackEffect::Unary(ty) => {
                        self.current_stack.pop();
                        self.current_stack.push(ty);
                    }
                    StackEffect::Binary(ty) => {
                        self.current_stack.pop();
                        self.current_stack.pop();
                        self.current_stack.push(ty);
                    }
                    StackEffect::Add(ty) => {
                        self.current_stack.push(ty);
                    }
                    StackEffect::Remove => {
                        self.current_stack.pop();
                    }
                    StackEffect::RemoveTwo => {
                        self.current_stack.pop();
                        self.current_stack.pop();
                    }
                }
                self.emit_instruction_from_current_function(i);
            }
        }
        self.emit_end_expression(MODULE_MEMBER_INDENT);
    }

    fn handle_export(&mut self, export: &'a Export) {
        self.emit_section(export.span.offset(), MODULE_MEMBER_INDENT);
    }

    fn handle_type(&mut self, _ty: &'a Type) {
        // Emitting the utx function type on setup breaks other function references,
        // therefore we don't emit them.
        // A possible fix is to parse the __step function

        //self.emit_section(ty.span.offset(), MODULE_MEMBER_INDENT);
    }

    fn finish_and_build_result(&mut self) -> Self::WalkResult {
        self.emit_memory();
        self.emit_funcref_table();
        self.emit_end_expression(MODULE_INDENT);
    }
}
