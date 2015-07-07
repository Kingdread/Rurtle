use super::{Environment, ResultType, RuntimeError, Value};
use std::fs;

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

pub fn screenshot(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref name), => {
        let shot = env.get_turtle().get_screen().screenshot();
        let mut file = match fs::File::create(name) {
            Ok(f) => f,
            Err(e) => return Err(RuntimeError(format!("{}", e))),
        };
        match shot.save(&mut file, ::image::ImageFormat::PNG) {
            Ok(()) => Ok(Value::Nothing),
            Err(e) => Err(RuntimeError(format!("{}", e))),
        }
    })
}

pub fn prompt(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref prompt_string), => {
        // What?
        let input = super::super::super::readline::readline(prompt_string);
        match input {
            Some(i) => Ok(Value::String(i)),
            None => Err(RuntimeError("No input to get".to_string())),
        }
    })
}

pub fn throw(_: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref error_desc), => {
        Err(RuntimeError(error_desc.clone()))
    })
}
