//! This module contains predefined functions for the Rurtle environment.
//!
//! All functions that should be "built in" into the Rurtle language (all the
//! fundamental stuff) are defined here or in submodules.
pub use super::{Environment, Function, ResultType, RuntimeError};
pub use super::value::Value;
use super::Function::Native;
use std::collections::HashMap;

// Example function.
//
// The first argument is the `Environment` in which the function is called, the
// second argument is are the (already evaluated) arguments.
fn print(_: &mut Environment, args: &[Value]) -> ResultType {
    println!("{}", args[0]);
    Ok(Value::Nothing)
}

/// Helper macro to extract the given arguments.
///
/// The first parameter is the slice of arguments to match against, followed by
/// a list of patterns delimited by , and prefixed by `arg`. Finally there is
/// the => expr part which specifies what should happen with the arguments.
///
/// If a argument can't be matched with the given pattern, a
/// `Err(RuntimeError(..))` is returned.
///
/// # Example
///
/// ```
/// fn three_args(_: &mut Environment, args: &[Value]) -> ResultType {
///     get_args!(args, arg Value::Number(a),
///                     arg Value::String(ref b),
///                     arg Value::List(ref c), => {
///     Ok(Value::Nothing)
/// })
/// }
/// ```
///
/// *Note*: There is a comma even behind the last pattern, don't forget it or
/// the compiler will spit fire at you!
macro_rules! get_args {
    ($args:expr, $(arg $ps:pat,)* => $b:expr) => {
        get_args!(index 0, $args, $(arg $ps,)* => $b)
    };

    (index $ind:expr, $args:expr, arg $p:pat, $(arg $ps:pat,)* => $b:expr) => {
        {
            let arg = &$args[$ind];
            if let $p = *arg {
                get_args!(index $ind+1, $args, $(arg $ps,)* => $b)
            } else {
                Err(RuntimeError(format!("invalid argument: {:?}", arg)))
            }
        }
    };

    (index $ind:expr, $args:expr, => $b:expr) => { $b };
}

mod turtle;
mod env;

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
        "PRINT" => Native(1, print),

        // Turtle control/draw functions
        "FORWARD" => Native(1, turtle::forward),
        "BACKWARD" => Native(1, turtle::backward),
        "LEFT" => Native(1, turtle::left),
        "RIGHT" => Native(1, turtle::right),
        "COLOR" => Native(3, turtle::color),
        "CLEAR" => Native(0, turtle::clear),
        "PENDOWN" => Native(0, turtle::pendown),
        "PENUP" => Native(0, turtle::penup),
        "HOME" => Native(0, turtle::home),
        "REALIGN" => Native(1, turtle::realign),

        // Environment functions to set variables
        "MAKE" => Native(2, env::make),
        "GLOBAL" => Native(2, env::global),
    }
}
