use super::{Environment, ResultType, RuntimeError, Value};

pub fn forward(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.get_turtle().forward(x);
        Ok(Value::Nothing)
    })
}

pub fn backward(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.get_turtle().backward(x);
        Ok(Value::Nothing)
    })
}

pub fn left(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.get_turtle().left(x);
        Ok(Value::Nothing)
    })
}

pub fn right(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.get_turtle().right(x);
        Ok(Value::Nothing)
    })
}

pub fn color(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::Number(a),
              arg Value::Number(b),
              arg Value::Number(c), => {
                  env.get_turtle().set_color(a, b, c);
                  Ok(Value::Nothing)
              })
}

pub fn bgcolor(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::Number(r),
              arg Value::Number(g),
              arg Value::Number(b), => {
                  env.get_turtle().set_background_color(r, g, b);
                  Ok(Value::Nothing)
              })
}

pub fn clear(env: &mut Environment, _: &[Value]) -> ResultType {
    env.get_turtle().clear();
    Ok(Value::Nothing)
}

pub fn pendown(env: &mut Environment, _: &[Value]) -> ResultType {
    env.get_turtle().pen_down();
    Ok(Value::Nothing)
}

pub fn penup(env: &mut Environment, _: &[Value]) -> ResultType {
    env.get_turtle().pen_up();
    Ok(Value::Nothing)
}

pub fn home(env: &mut Environment, _: &[Value]) -> ResultType {
    env.get_turtle().home();
    Ok(Value::Nothing)
}

pub fn realign(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.get_turtle().set_orientation(x);
        Ok(Value::Nothing)
    })
}

pub fn hide(env: &mut Environment, _: &[Value]) -> ResultType {
    env.get_turtle().hide();
    Ok(Value::Nothing)
}

pub fn show(env: &mut Environment, _: &[Value]) -> ResultType {
    env.get_turtle().show();
    Ok(Value::Nothing)
}

pub fn write(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref s), => {
        env.get_turtle().write(s);
        Ok(Value::Nothing)
    })
}

pub fn flood(env: &mut Environment, _: &[Value]) -> ResultType {
    env.get_turtle().flood();
    Ok(Value::Nothing)
}

pub fn procreate(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref s), => {
        if env.add_turtle(s.clone()) {
            Ok(Value::Nothing)
        } else {
            Err(RuntimeError("That turtle already exists".into()))
        }
    })
}

pub fn select(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref s), => {
        if env.select_turtle(s) {
            Ok(Value::Nothing)
        } else {
            Err(RuntimeError("That turtle does not exist".into()))
        }
    })
}

pub fn delete(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::String(ref s), => {
        if env.delete_turtle(s) {
            Ok(Value::Nothing)
        } else {
            Err(RuntimeError("That turtle can't be deleted".into()))
        }
    })
}
