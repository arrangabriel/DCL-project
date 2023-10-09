use wast::core::{Instruction, ValType};

#[derive(Debug)]
pub struct Module<'a> {
    pub functions: Box<[Function<'a>]>,
}

#[derive(Debug)]
pub struct Function<'a> {
    pub id: Option<String>,
    pub signature: Signature<'a>,
    pub instructions: &'a [Instruction<'a>],
}

impl<'a> Function<'a> {
    pub fn shallow_clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            signature: self.signature.clone(),
            instructions: self.instructions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Signature<'a> {
    pub parameters: Vec<FuncParameter<'a>>,
    pub results: Vec<ValType<'a>>,
}

#[derive(Debug, Clone)]
pub struct FuncParameter<'a> {
    pub id: Option<&'a str>,
    pub val_type: &'a ValType<'a>,
}
