pub use function::IGNORE_FUNC_PREFIX;
pub use transform::emit_transformed_wat;
pub use instruction::{InstructionType, MemoryInstructionType};

mod emit;
mod function;
mod instruction;
mod instruction_stream;
mod split;
mod transform;
mod utils;
