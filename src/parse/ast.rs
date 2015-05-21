#[derive(Debug, Clone)]
pub enum Node {
    /// A list of statements as found inside a loop body
    StatementList(Vec<Box<Node>>),
    /// The if conditional (expression, true-clause, maybe false-clause)
    IfStatement(Box<Node>, Box<Node>, Option<Box<Node>>),
    /// The repeat statement (count, loop body)
    RepeatStatement(Box<Node>, Box<Node>),
    /// The while statement (condition, loop body)
    WhileStatement(Box<Node>, Box<Node>),
    /// The function definition statement (func name, func arg names, func body)
    LearnStatement(String, Vec<String>, Box<Node>),
    Comparison(Box<Node>, CompOp, Box<Node>),
    /// Addition or subtraction. One addition may hold more than one operation.
    Addition(Box<Node>, Vec<(AddOp, Box<Node>)>),
    /// Multiplication and division. One multiplication may hole more than one
    /// operation.
    Multiplication(Box<Node>, Vec<(MulOp, Box<Node>)>),
    /// A function call (function, arguments)
    FuncCall(String, Vec<Box<Node>>),
    ReturnStatement(Box<Node>),
    List(Vec<Box<Node>>),
    StringLiteral(String),
    Number(f32),
    Variable(String),
}

/// Helper function to flatten a vector of boxes to nodes
fn flatten(mut input: Vec<Box<Node>>) -> Vec<Box<Node>> {
    let mut result = Vec::new();
    while !input.is_empty() {
        result.push(Box::new(input.remove(0).flatten()));
    }
    result
}

fn flatten_tuple<T>(mut input: Vec<(T, Box<Node>)>) -> Vec<(T, Box<Node>)> {
    let mut result = Vec::new();
    while !input.is_empty() {
        let (op, elem) = input.remove(0);
        result.push((op, Box::new(elem.flatten())));
    }
    result
}

impl Node {
    /// Consume the node and produce a flat version
    pub fn flatten(self) -> Node {
        use self::Node::*;
        match self {
            Addition(sum, summands) => {
                if summands.is_empty() {
                    sum.flatten()
                } else {
                    Addition(Box::new(sum.flatten()), flatten_tuple(summands))
                }
            },
            Multiplication(mul, factors) => {
                if factors.is_empty() {
                    mul.flatten()
                } else {
                    Multiplication(Box::new(mul.flatten()), flatten_tuple(factors))
                }
            }
            StatementList(mut stmts) => {
                if stmts.len() == 1 {
                    stmts.remove(0).flatten()
                } else {
                    StatementList(flatten(stmts))
                }
            },
            List(elements) => List(flatten(elements)),
            IfStatement(cond, true_body, false_body) => {
                if let Some(stmt) = false_body {
                    IfStatement(Box::new(cond.flatten()), Box::new(true_body.flatten()),
                                Some(Box::new(stmt.flatten())))
                } else {
                    IfStatement(Box::new(cond.flatten()), Box::new(true_body.flatten()), None)
                }
            },
            RepeatStatement(count, body) => RepeatStatement(Box::new(count.flatten()),
                                                            Box::new(body.flatten())),
            WhileStatement(cond, body) => WhileStatement(Box::new(cond.flatten()),
                                                         Box::new(body.flatten())),
            LearnStatement(name, args, body) => LearnStatement(name, args,
                                                               Box::new(body.flatten())),
            Comparison(operand1, op, operand2) => Comparison(Box::new(operand1.flatten()),
                                                             op,
                                                             Box::new(operand2.flatten())),
            ReturnStatement(value) => ReturnStatement(Box::new(value.flatten())),
            FuncCall(name, args) => FuncCall(name, flatten(args)),
            node => node,
        }
    }
}

/// Different comparison operators
#[derive(Debug, Copy, Clone)]
pub enum CompOp {
    Equal, Less, Greater, LessEqual, GreaterEqual, NotEqual,
}

#[derive(Debug, Copy, Clone)]
pub enum AddOp { Add, Sub }
#[derive(Debug, Copy, Clone)]
pub enum MulOp { Mul, Div }
