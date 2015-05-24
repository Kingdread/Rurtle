use super::{Environment, Function, ResultType, RuntimeError};
use super::value::Value;
use super::Function::Native;
use std::collections::HashMap;

// Example function.
//
// The first argument is the `Environment` in which the function is called, the
// second argument is are the (already evaluated) arguments.
fn hello_world(_: &mut Environment, args: &[Value]) -> ResultType {
    match args[0] {
        Value::String(ref s) => {
            println!("Hello, {}", s);
            Ok(Value::Nothing)
        }
        _ => Err(RuntimeError("Hello must be called with a string".to_string())),
    }
}

/// A helpful macro to construct a `HashMap`
macro_rules! map {
    ($($k:expr => $v:expr,) *) => {
        {
            let mut result = HashMap::new();
            $(result.insert($k.to_string(), $v);)*
            result
        }
    }
}

/// Return a HashMap of the built-in functions
pub fn default_functions() -> HashMap<String, Function> {
    map!{
        "HELLO" => Native(1, hello_world),
    }
}
