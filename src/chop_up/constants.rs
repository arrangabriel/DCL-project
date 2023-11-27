use crate::chop_up::instruction::DataType;

pub const UTX_LOCALS: [DataType; 3] = [DataType::I32, DataType::I32, DataType::I32];
pub const ADDRESS_LOCAL_NAME: &str = "memory_address";
pub const STACK_JUGGLER_NAME: &str = "local";
pub const MODULE_MEMBER_INDENT: usize = 1;