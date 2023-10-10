pub use marking::is_mem_access_instruction;
pub use print::{print_functions, print_with_safety};
pub use split::split_unsafe_functions;

mod marking;
mod print;
mod split;
