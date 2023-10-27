use rand::distributions::Alphanumeric;
use rand::Rng;

pub const TRANSACTION_FUNCTION_SIGNATURE: &str =
    "(type $utx_f) (param $tx i32) (param $utx i32) (param $state i32) (result i32)";
pub const IGNORE_FUNC_PREFIX: &str = "__";
pub const ADDRESS_LOCAL_NAME: &str = "memory_address";
pub const VALUE_LOCAL_NAME: &str = "value_to_store";
pub const STACK_SAVE_NAME: &str = "stack_local";
pub const INSTRUCTION_INDENT: usize = 2;
pub const MODULE_MEMBER_INDENT: usize = 1;
pub const MODULE_INDENT: usize = 0;
pub const INDENTATION_STR: &str = "    ";

pub fn gen_random_func_name() -> String {
    let rand_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect();
    format!("funcid_{rand_id}")
}
