use wast::core::{Instruction, ValType};

#[derive(Debug)]
pub struct Module<'a> {
    pub(crate) functions: Box<[Function<'a>]>,
}

#[derive(Debug)]
pub struct Function<'a> {
    pub(crate) id: Option<String>,
    pub(crate) signature: Signature<'a>,
    pub(crate) instructions: &'a [Instruction<'a>],
}

#[derive(Debug)]
pub struct Signature<'a> {
    pub(crate) parameters: Vec<FuncParameter<'a>>,
    pub(crate) results: Vec<ValType<'a>>,
}

#[derive(Debug)]
pub struct FuncParameter<'a> {
    pub(crate) id: Option<&'a str>,
    pub(crate) val_type: &'a ValType<'a>,
}
