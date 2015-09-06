use super::{Environment, ResultType, RuntimeError, Value};

pub fn replace(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::String(ref haystack),
              arg Value::String(ref needle),
              arg Value::String(ref replacement), =>
    {
        Ok(Value::String(haystack.replace(needle, replacement)))
    })
}

pub fn contains(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::String(ref original),
              arg Value::String(ref pattern), =>
    {
        Ok(Value::Number(if original.contains(pattern) { 1. } else { 0. }))
    })
}

pub fn chars(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref string), => {
        Ok(Value::List(string.chars().map(|c| Value::String(c.to_string())).collect()))
    })
}

pub fn split(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::String(ref string),
              arg Value::String(ref pattern), =>
    {
        Ok(Value::List(string.split(pattern).map(|s| Value::String(s.to_owned())).collect()))
    })
}
