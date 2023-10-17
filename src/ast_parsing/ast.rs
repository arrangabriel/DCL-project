use wast::core::{
    Func, FuncKind, FunctionType, Instruction, Memory, Module, ModuleField, ModuleKind,
};
use wast::Wat;

/// Implementers may themselves decide on the granularity of ast they want to handle.
///
/// For functions handler order will always be
/// 1. [handle_func]
/// 2. [handle_func_type]
/// 3. [handle_func_instructions]
pub trait AstWalker<'a> {
    type WalkResult;
    fn handle_module(&mut self, _module: &Module) {}
    fn handle_fields(&mut self, _fields: &[ModuleField]) {}
    fn handle_func(&mut self, _func: &'a Func, _instructions: &'a [Instruction]) {}
    fn start_handle_func(&mut self, _func: &'a Func) {}
    fn handle_func_type(&mut self, _func_type: &'a FunctionType) {}
    fn handle_func_instructions(&mut self, _instructions: &'a [Instruction]) {}
    fn handle_memory(&mut self, _memory: &'a Memory) {}
    fn finish_and_build_result(&mut self) -> Self::WalkResult;
}

pub fn walk_ast<'a, T: 'a>(
    wat: &'a Wat,
    mut ast_walker: Box<(dyn AstWalker<'a, WalkResult = T> + 'a)>,
) -> T {
    match wat {
        Wat::Module(module) => {
            ast_walker.handle_module(module);
            match &module.kind {
                ModuleKind::Text(module_fields) => {
                    ast_walker.handle_fields(module_fields);
                    for field in module_fields {
                        match field {
                            ModuleField::Func(func) => {
                                ast_walker.start_handle_func(func);
                                if let Some(function_type) = &func.ty.inline {
                                    ast_walker.handle_func_type(function_type);
                                }
                                match &func.kind {
                                    FuncKind::Inline {
                                        locals: _locals,
                                        expression,
                                    } => {
                                        ast_walker.handle_func_instructions(&expression.instrs);
                                        ast_walker.handle_func(func, &expression.instrs);
                                    }
                                    FuncKind::Import(_) => {}
                                }
                            }
                            ModuleField::Type(_) => {}
                            ModuleField::Rec(_) => {}
                            ModuleField::Import(_) => {}
                            ModuleField::Table(_) => {}
                            ModuleField::Memory(memory) => ast_walker.handle_memory(memory),
                            ModuleField::Global(_) => {}
                            ModuleField::Export(_) => {}
                            ModuleField::Start(_) => {}
                            ModuleField::Elem(_) => {}
                            ModuleField::Data(_) => {}
                            ModuleField::Tag(_) => {}
                            ModuleField::Custom(_) => {}
                        }
                    }
                }
                ModuleKind::Binary(_) => {}
            }
        }
        Wat::Component(_) => {}
    }
    ast_walker.finish_and_build_result()
}
