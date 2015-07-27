//! Data and methods for executing Rurtle code.
//!
//! Parsing and lexing input is fine but sooner or later you need to actually
//! execute the parsed tree. This module contains `Environment`, the execution
//! environment for Rurtle code. It also defines the "prelude", the built-in
//! functions in the Rurtle language.
pub mod functions;
pub mod value;
pub mod stack;
use self::value::Value;
use super::parse::ast::{Node, AddOp, MulOp, CompOp};
use super::turtle;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct RuntimeError(String);

impl ::std::fmt::Display for RuntimeError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        fmt.pad(&self.0)
    }
}

impl ::std::error::Error for RuntimeError {
    fn description(&self) -> &str {
        "runtime error"
    }
}

/// The type returned by Rurtle functions
pub type ResultType = Result<Value, RuntimeError>;
/// The type that functions called in Rurtle must have.
///
/// The first parameter is the Environment in which the function is executed and
/// the second argument are the function's parameters.
pub type FuncType = fn(&mut Environment, &[Value]) -> ResultType;

/// A function available to Rurtle programs can either be a function defined in
/// a Rurtle program or a native function of FuncType
pub enum Function {
    /// This variant holds a function that was defined in Rurtle via the LEARN
    /// statement. The node passed has to be the `LearnStatement` node.
    Defined(Node),
    /// This variant holds a native Rust function that should be available to
    /// Rurtle. The first parameter is the number of arguments since they can't
    /// be extracted from the function pointer. The second argument is a
    /// function of `FuncType`
    Native(i32, FuncType),
}

impl Clone for Function {
    fn clone(&self) -> Function {
        use self::Function::*;
        match *self {
            Defined(ref node) => Defined(node.clone()),
            Native(arg_count, function) => Native(arg_count, function),
        }
    }
}

/// Helper function to get a pointer without needing to type the type
fn pointer<T>(x: &T) -> *const T { x as *const T }

impl fmt::Debug for Function {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Function::*;
        match *self {
            Defined(ref node) => {
                write!(fmt, "Defined({:?})", node)
            },
            Native(count, function) => {
                write!(fmt, "Native({:?}, {:?})", count, pointer(&function))
            },
        }
    }
}

macro_rules! framed {
    ($s:expr, $what:expr) => {
        {
            $s.push_inner_frame();
            let result = $what;
            $s.pop_inner_frame();
            result
        }
    }
}

pub struct Environment {
    stack: Vec<stack::Frame>,
    turtle: turtle::Turtle,
}

impl Environment {
    /// Construct a new `Environment` with default values
    pub fn new(turtle: turtle::Turtle) -> Environment {
        Environment {
            stack: stack::new_stack(),
            turtle: turtle,
        }
    }

    pub fn get_turtle(&mut self) -> &mut turtle::Turtle {
        &mut self.turtle
    }

    /// Return a map mapping the function name to the argument count. Useful for
    /// passing it to `Parser::parse`
    pub fn function_arg_count(&self) -> HashMap<String, i32> {
        let mut result = HashMap::new();
        // We walk the stack and insert the functions of each frame into the
        // "global stack" of the parser. If a function is redefined in a "tighter"
        // stack, it will overwrite the more general version
        for stack_frame in &self.stack {
            for mini_frame in &stack_frame.functions {
                for (name, function) in mini_frame {
                    let count = match *function {
                        Function::Native(i, _) => i,
                        Function::Defined(ref node) => {
                            match *node {
                                Node::LearnStatement(_, ref args, _) => args.len() as i32,
                                _ => panic!("Function node is not a LearnStatement"),
                            }
                        },
                    };
                    result.insert(name.clone(), count);
                }
            }
        }
        result
    }

    fn find_function(&self, name: &str) -> Option<&Function> {
        for stack_frame in self.stack.iter().rev() {
            for mini_frame in stack_frame.functions.iter().rev() {
                match mini_frame.get(name) {
                    Some(f) => return Some(f),
                    None => (),
                }
            }
        }
        None
    }

    /// Tokenize, parse and evaluate the given source
    pub fn eval_source(&mut self, source: &str) -> Result<Value, Box<::std::error::Error>> {
        use super::lex;
        use super::parse;
        let tokens = match lex::tokenize(source) {
            Ok(t) => t,
            Err(e) => return Err(Box::new(e)),
        };
        let mut parser = parse::Parser::new(tokens, self.function_arg_count());
        let tree = match parser.parse() {
            Ok(n) => n.flatten(),
            Err(e) => return Err(Box::new(e)),
        };
        match self.eval(&tree) {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Box::new(e)),
        };
    }

    /// Evaluate the given AST node
    pub fn eval(&mut self, node: &Node) -> ResultType {
        use super::parse::ast::Node::*;
        if self.current_frame().should_return {
            return Ok(Value::Nothing);
        }
        match *node {
            StatementList(ref nodes) =>
                self.eval_statement_list(nodes),
            IfStatement(ref condition, ref true_body, ref false_body) =>
                self.eval_if_statement(condition, true_body, false_body),
            RepeatStatement(ref num, ref body) =>
                self.eval_repeat_statement(num, body),
            WhileStatement(ref condition, ref body) =>
                self.eval_while_statement(condition, body),
            ref learn_statement @ LearnStatement(..) =>
                self.eval_learn_statement(learn_statement),
            Comparison(ref a, op, ref b) =>
                self.eval_comparison(a, op, b),
            Addition(ref start, ref values) =>
                self.eval_addition(start, values),
            Multiplication(ref start, ref values) =>
                self.eval_multiplication(start, values),
            FuncCall(ref name, ref args) =>
                self.eval_func_call(name, args),
            ReturnStatement(ref value) =>
                self.eval_return_statement(value),
            TryStatement(ref normal, ref exception) =>
                self.eval_try_statement(normal, exception),
            List(ref elements) =>
                self.eval_list(elements),
            StringLiteral(ref string) =>
                Ok(Value::String(string.clone())),
            Number(num) =>
                Ok(Value::Number(num)),
            Variable(ref name) =>
                self.eval_variable(name),
        }
    }

    fn eval_statement_list(&mut self, statements: &Vec<Node>) -> ResultType {
        for statement in statements {
            try!(self.eval(statement));
        }
        Ok(Value::Nothing)
    }

    fn eval_if_statement(&mut self, condition: &Node, true_body: &Node,
                         false_body: &Option<Box<Node>>)
                         -> ResultType
    {
        let value = try!(self.eval(condition));
        if value.boolean() {
            try!(framed!(self, self.eval(true_body)));
        } else if let &Some(ref false_body) = false_body {
            try!(framed!(self, self.eval(false_body)));
        }
        Ok(Value::Nothing)
    }

    fn eval_repeat_statement(&mut self, num: &Node, body: &Node) -> ResultType {
        let num = try!(self.eval(num));
        if let Value::Number(num) = num {
            for _ in (0..num as i32) {
                try!(framed!(self, self.eval(body)));
            }
            Ok(Value::Nothing)
        } else {
            Err(RuntimeError("repeat count has to be a number".to_string()))
        }
    }

    fn eval_while_statement(&mut self, condition: &Node, body: &Node) -> ResultType {
        while try!(self.eval(condition)).boolean() {
            try!(framed!(self, self.eval(body)));
        }
        Ok(Value::Nothing)
    }

    fn eval_learn_statement(&mut self, statement: &Node) -> ResultType {
        if let Node::LearnStatement(ref name, _, _) = *statement {
            self.current_frame().functions.last_mut().unwrap()
                .insert(name.clone(), Function::Defined(statement.clone()));
            Ok(Value::Nothing)
        } else {
            panic!("{:?} is not a LearnStatement", statement);
        }
    }

    fn eval_try_statement(&mut self, normal: &Node, exception: &Node) -> ResultType {
        let result = framed!(self, self.eval(normal));
        match result {
            Ok(_) => Ok(Value::Nothing),
            Err(_) => {
                framed!(self, self.eval(exception))
            },
        }
    }

    fn eval_comparison(&mut self, a: &Node, op: CompOp, b: &Node) -> ResultType {
        let value_a = try!(self.eval(a));
        let value_b = try!(self.eval(b));
        let compare = value_a.partial_cmp(&value_b);
        match compare {
            Some(ordering) => Ok(Value::Number({
                if op.matches(&ordering) { 1.0 } else { 0.0 }
            })),
            None => Err(RuntimeError(format!("Can't compare {} and {}",
                                             value_a.type_string(), value_b.type_string()))),
        }
    }

    fn eval_addition(&mut self, start: &Node, values: &Vec<(AddOp, Node)>) -> ResultType {
        let mut accum = try!(self.eval(start));
        for &(op, ref value) in values.iter() {
            let value = try!(self.eval(value));
            let result = match op {
                AddOp::Add => &accum + &value,
                AddOp::Sub => &accum - &value,
            };
            accum = match result {
                Some(v) => v,
                None => return Err(RuntimeError(
                    format!("Can't add/subtract {} and {}",
                            accum.type_string(), value.type_string()))),
            }
        }
        Ok(accum)
    }

    fn eval_multiplication(&mut self, start: &Node, values: &Vec<(MulOp, Node)>) -> ResultType {
        let mut accum = try!(self.eval(start));
        for &(op, ref value) in values.iter() {
            let value = try!(self.eval(value));
            let result = match op {
                MulOp::Mul => &accum * &value,
                MulOp::Div => &accum / &value,
            };
            accum = match result {
                Some(v) => v,
                None => return Err(RuntimeError(
                    format!("Can't multiply/divide {} and {}",
                            accum.type_string(), value.type_string()))),
            }
        }
        Ok(accum)
    }

    fn eval_func_call(&mut self, name: &String, arg_nodes: &Vec<Node>) -> ResultType {
        let function = match self.find_function(&name.to_uppercase()) {
            Some(f) => f.clone(),
            None => return Err(RuntimeError(format!("function {} not found", name))),
        };
        let args: Vec<Value> = try!(arg_nodes.iter().map(|a| self.eval(a)).collect());
        match function {
            Function::Native(_, ref f) => {
                f(self, &args)
            },
            Function::Defined(ref node) => {
                match *node {
                    Node::LearnStatement(ref name, ref arg_names, ref body) =>
                        self.call_defined_function(name, arg_names, args, body),
                    _ => panic!("Defined function is no LearnStatement"),
                }
            }
        }
    }

    fn call_defined_function(&mut self, name: &String, arg_names: &Vec<String>,
                             args: Vec<Value>, body: &Node)
                             -> ResultType
    {
        let mut frame = stack::Frame::default();
        frame.fn_name = name.clone();
        for (name, value) in arg_names.iter().zip(args) {
            frame.locals.insert(name.clone(), value);
        }
        self.stack.push(frame);
        let result = self.eval(body);
        frame = self.stack.pop().unwrap();
        try!(result);
        match frame.return_value {
            Some(value) => Ok(value),
            None => Ok(Value::Nothing),
        }
    }

    fn eval_return_statement(&mut self, value: &Node) -> ResultType {
        if self.current_frame().is_global {
            return Err(RuntimeError("Return not in a function".to_string()));
        }
        let value = try!(self.eval(value));
        self.current_frame().return_value = Some(value);
        self.current_frame().should_return = true;
        Ok(Value::Nothing)
    }

    fn eval_list(&mut self, elements: &Vec<Node>) -> ResultType {
        let mut result = Vec::new();
        for node in elements {
            result.push(try!(self.eval(node)));
        }
        Ok(Value::List(result))
    }

    fn eval_variable(&mut self, name: &String) -> ResultType {
        match self.get_variable(name) {
            Some(value) => Ok(value),
            None => Err(RuntimeError(format!("Variable {} not found", name))),
        }
    }

    /// Return the current stack frame or the global frame if the code is not
    /// executing in a function
    pub fn current_frame(&mut self) -> &mut stack::Frame {
        let len = self.stack.len();
        &mut self.stack[len - 1]
    }

    /// Return the global frame
    pub fn global_frame(&mut self) -> &mut stack::Frame {
        &mut self.stack[0]
    }

    fn push_inner_frame(&mut self) {
        self.current_frame().functions.push(HashMap::new());
    }

    fn pop_inner_frame(&mut self) {
        self.current_frame().functions.pop().unwrap();
    }

    /// Retrieve the value for the variable with the given name
    ///
    /// The variable is searched in the current function's local variables. If
    /// it is not defined there, the global namespace will be searched. If the
    /// variable is not found there either, `None` is returned.
    pub fn get_variable(&mut self, name: &str) -> Option<Value> {
        {
            let local_frame = self.current_frame();
            match local_frame.locals.get(name) {
                Some(value) => return Some(value.clone()),
                None => (),
            }
        }
        let global_frame = self.global_frame();
        global_frame.locals.get(name).map(|v| v.clone())
    }
}
