use super::{Environment, ResultType, RuntimeError, Value};

pub fn make(env: &mut Environment, args: &[Value]) -> ResultType {
    if let Value::String(ref name) = args[0] {
        env.current_frame().locals.insert(name.clone(), args[1].clone());
        Ok(Value::Nothing)
    } else {
        Err(RuntimeError(format!("invalid argument: {:?}", args[1])))
    }
}

pub fn global(env: &mut Environment, args: &[Value]) -> ResultType {
    if let Value::String(ref name) = args[0] {
        env.global_frame().locals.insert(name.clone(), args[1].clone());
        Ok(Value::Nothing)
    } else {
        Err(RuntimeError(format!("invalid argument: {:?}", args[1])))
    }
}
