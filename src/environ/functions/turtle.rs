use super::{Environment, ResultType, RuntimeError, Value};

pub fn forward(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.forward(x);
        Ok(Value::Nothing)
    })
}

pub fn backward(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.backward(x);
        Ok(Value::Nothing)
    })
}

pub fn left(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.left(x);
        Ok(Value::Nothing)
    })
}

pub fn right(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.right(x);
        Ok(Value::Nothing)
    })
}

pub fn color(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args,
              arg Value::Number(a),
              arg Value::Number(b),
              arg Value::Number(c), => {
                  env.turtle.set_color(a, b, c);
                  Ok(Value::Nothing)
              })
}

pub fn clear(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.clear();
    Ok(Value::Nothing)
}

pub fn pendown(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.pen_down();
    Ok(Value::Nothing)
}

pub fn penup(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.pen_up();
    Ok(Value::Nothing)
}

pub fn home(env: &mut Environment, _: &[Value]) -> ResultType {
    env.turtle.home();
    Ok(Value::Nothing)
}

pub fn realign(env: &mut Environment, args: &[Value]) -> ResultType {
    get_args!(args, arg Value::Number(x), => {
        env.turtle.set_orientation(x);
        Ok(Value::Nothing)
    })
}
