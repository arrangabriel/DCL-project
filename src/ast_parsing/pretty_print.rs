use wast::core::{Func, FunctionType, Instruction, Module, ModuleField, ValType};
use wast::token::{Id, NameAnnotation};
use wast::Wat;

use crate::ast_parsing::ast::{walk_ast, AstWalker};

/// Print a wasm module
pub fn pretty_print_ast(ast: &Wat) {
    walk_ast(ast, Box::new(AstPrettyFormatter::new()));
}

struct AstPrettyFormatter {
    ast_string: String,
}

impl AstPrettyFormatter {
    fn new() -> Self {
        AstPrettyFormatter {
            ast_string: String::new(),
        }
    }

    fn push_line(&mut self, line: &str, indent: u8) {
        if !self.ast_string.is_empty() {
            self.ast_string.push('\n');
        }
        self.ast_string
            .push_str(&String::from(' ').repeat((indent * 2) as usize));
        self.ast_string.push_str(line);
    }
}

impl AstWalker<'_> for AstPrettyFormatter {
    type WalkResult = ();

    fn handle_module(&mut self, module: &Module) {
        self.push_line("Module:", 0);
        self.push_line(&format!("name - {:?}", module.name), 1);
        self.push_line(&format!("id - {:?}", module.id), 1);
    }

    fn handle_fields(&mut self, _: &[ModuleField]) {
        self.push_line("Fields:", 1);
    }

    fn start_handle_func(&mut self, func: &Func) {
        self.push_line("Function:", 2);
        if let Some(id) = func.id {
            self.push_line(&format!("id - {:?}", id), 3);
        }
    }

    fn handle_func_type(&mut self, func_type: &FunctionType) {
        let nesting = 3;
        let formatted_params = format_params(&func_type.params);
        self.push_line(&format!("params - {:?}", formatted_params), nesting);
        if func_type.results.len() == 1 {
            self.push_line(&format!("result - {:?}", func_type.results[0]), nesting);
        } else {
            self.push_line(&format!("results - {:?}", func_type.results), nesting);
        }
    }

    fn handle_func_instructions(&mut self, instructions: &[Instruction]) {
        let nesting = 3;
        self.push_line("instructions:", nesting);
        for instruction in instructions {
            self.push_line(&format!("{:?}", instruction), nesting + 1);
        }
    }

    fn finish_and_build_result(&mut self) -> Self::WalkResult {
        println!("{}", self.ast_string);
        ()
    }
}

fn format_params(params: &[(Option<Id>, Option<NameAnnotation>, ValType)]) -> Vec<String> {
    params
        .iter()
        .map(|param| {
            let mut param_repr = String::new();
            if let Some(name) = param.0 {
                param_repr.push_str(name.name());
                param_repr.push_str(" - ");
            }
            param_repr.push_str(&*format!("{:?}", param.2));
            param_repr
        })
        .collect()
}
