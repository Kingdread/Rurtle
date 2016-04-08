//! This module contains predefined functions for the Rurtle environment.
//!
//! All functions that should be "built in" into the Rurtle language (all the
//! fundamental stuff) are defined here or in submodules.

// We "need" identity_op for the get_args! macro since it will expand to 0+1
// at some point and clippy doesn't like that
#![cfg_attr(feature = "linted", allow(identity_op))]

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
mod types;
mod string;

/// A helpful macro to construct a `HashMap`
macro_rules! map {
    ($($k:expr => $v:expr,) *) => {
        {
            let mut result = HashMap::new();
            $(result.insert($k.to_owned(), $v);)*
            result
        }
    }
}

/// Return a `HashMap` of the built-in functions
pub fn default_functions() -> HashMap<String, Function> {
    map!{
        "PRINT" => Native(1, print),

        // Turtle control/draw functions
        "FORWARD" => Native(1, turtle::forward),
        "BACKWARD" => Native(1, turtle::backward),
        "LEFT" => Native(1, turtle::left),
        "RIGHT" => Native(1, turtle::right),
        "COLOR" => Native(3, turtle::color),
        "BGCOLOR" => Native(3, turtle::bgcolor),
        "CLEAR" => Native(0, turtle::clear),
        "PENDOWN" => Native(0, turtle::pendown),
        "PENUP" => Native(0, turtle::penup),
        "HOME" => Native(0, turtle::home),
        "REALIGN" => Native(1, turtle::realign),
        "HIDE" => Native(0, turtle::hide),
        "SHOW" => Native(0, turtle::show),
        "WRITE" => Native(1, turtle::write),
        "FLOOD" => Native(0, turtle::flood),

        // Environment functions to set variables
        "MAKE" => Native(2, env::make),
        "GLOBAL" => Native(2, env::global),
        // Other environment functions
        "SCREENSHOT" => Native(1, env::screenshot),
        "PROMPT" => Native(1, env::prompt),
        "THROW" => Native(1, env::throw),

        // Haskellesque names
        "HEAD" => Native(1, types::head),
        "TAIL" => Native(1, types::tail),
        // Logo (alias) names
        "FIRST" => Native(1, types::head),
        "BUTFIRST" => Native(1, types::tail),
        // other list functions
        "LENGTH" => Native(1, types::length), // also works for strings
        "ISEMPTY" => Native(1, types::isempty),
        "GETINDEX" => Native(2, types::getindex),
        "FIND" => Native(2, types::find),
        // conversion
        "NOT" => Native(1, types::not),
        "TONUMBER" => Native(1, types::tonumber),
        "TOSTRING" => Native(1, types::tostring),
        "NOTHING" => Native(0, types::nothing),

        // String manipulating functions
        "REPLACE" => Native(3, string::replace),
        "CONTAINS" => Native(2, string::contains),
        "CHARS" => Native(1, string::chars),
        "SPLIT" => Native(2, string::split),
    }
}
