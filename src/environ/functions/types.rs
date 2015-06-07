use super::{Environment, ResultType, RuntimeError, Value};

pub fn head(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::List(ref values), => {
        if values.is_empty() {
            Ok(Value::Nothing)
        } else {
            Ok(values[0].clone())
        }
    })
}

pub fn tail(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::List(ref values), => {
        if values.is_empty() {
            Ok(Value::Nothing)
        } else {
            Ok(Value::List(values[1..].iter().map(|v| v.clone()).collect()))
        }
    })
}

pub fn length(_: &mut Environment, args: &[Value]) -> ResultType {
    match args[0] {
        Value::List(ref l) => Ok(Value::Number(l.len() as f32)),
        Value::String(ref s) => Ok(Value::Number(s.len() as f32)),
        ref val => Err(RuntimeError(format!("Invalid argument: {}", val))),
    }
}

pub fn isempty(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::List(ref values), => {
        Ok(Value::Number(if values.is_empty() { 1. } else { 0. }))
    })
}

pub fn getindex(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::List(ref values),
              arg Value::Number(n), =>
    {
        let idx = n as usize;
        if idx >= values.len() {
            Err(RuntimeError(format!("Index out of bounds: {} >= {}", idx, values.len())))
        } else {
            Ok(values[idx].clone())
        }
    })
}

pub fn find(_: &mut Environment, args: &[Value]) -> ResultType {
    if let Value::List(ref values) = args[0] {
        let needle = &args[1];
        for (i, hay) in values.iter().enumerate() {
            if hay == needle {
                return Ok(Value::Number(i as f32))
            }
        }
        Ok(Value::Number(-1.))
    } else {
        Err(RuntimeError(format!("Invalid argument: {}", args[0])))
    }
}

pub fn not(_: &mut Environment, args: &[Value]) -> ResultType {
    let as_boolean = args[0].boolean();
    Ok(Value::Number(if as_boolean { 0. } else { 1. }))
}

// Type conversion functions

pub fn tonumber(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref string), => {
        match string.parse::<f32>() {
            Ok(num) => Ok(Value::Number(num)),
            Err(e) => Err(RuntimeError(format!("{}", e))),
        }
    })
}

pub fn tostring(_: &mut Environment, args: &[Value]) -> ResultType {
    Ok(Value::String(format!("{}", args[0])))
}

pub fn nothing(_: &mut Environment, _: &[Value]) -> ResultType {
    Ok(Value::Nothing)
}
