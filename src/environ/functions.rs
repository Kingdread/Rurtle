use super::{Environment, Function, ResultType, RuntimeError};
use super::value::Value;
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

fn turtle_forward(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.forward(x);
        Ok(Value::Nothing)
    })
}

fn turtle_backward(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.backward(x);
        Ok(Value::Nothing)
    })
}

fn turtle_left(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.left(x);
        Ok(Value::Nothing)
    })
}

fn turtle_right(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.right(x);
        Ok(Value::Nothing)
    })
}

fn turtle_color(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::Number(a),
              arg Value::Number(b),
              arg Value::Number(c), => {
                  env.turtle.set_color(a, b, c);
                  Ok(Value::Nothing)
              })
}

fn turtle_clear(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.clear();
    Ok(Value::Nothing)
}

fn turtle_pendown(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.pen_down();
    Ok(Value::Nothing)
}

fn turtle_penup(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.pen_up();
    Ok(Value::Nothing)
}

fn turtle_home(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.home();
    Ok(Value::Nothing)
}

fn turtle_realign(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.set_orientation(x);
        Ok(Value::Nothing)
    })
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
        "PRINT" => Native(1, print),

        // Turtle control/draw functions
        "FORWARD" => Native(1, turtle_forward),
        "BACKWARD" => Native(1, turtle_backward),
        "LEFT" => Native(1, turtle_left),
        "RIGHT" => Native(1, turtle_right),
        "COLOR" => Native(3, turtle_color),
        "CLEAR" => Native(0, turtle_clear),
        "PENDOWN" => Native(0, turtle_pendown),
        "PENUP" => Native(0, turtle_penup),
        "HOME" => Native(0, turtle_home),
        "REALIGN" => Native(1, turtle_realign),
    }
}
