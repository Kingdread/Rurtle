#[derive(Debug, Clone)]
pub enum Node {
    /// A list of statements as found inside a loop body
    StatementList(Vec<Node>),
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
    Addition(Box<Node>, Vec<(AddOp, Node)>),
    /// Multiplication and division. One multiplication may hole more than one
    /// operation.
    Multiplication(Box<Node>, Vec<(MulOp, Node)>),
    /// A function call (function, arguments)
    FuncCall(String, Vec<Node>),
    ReturnStatement(Box<Node>),
    List(Vec<Node>),
    StringLiteral(String),
    Number(f32),
    Variable(String),
}

/// Helper function to flatten a vector of boxes to nodes
fn flatten(input: Vec<Node>) -> Vec<Node> {
    input.into_iter().map(|n| n.flatten()).collect()
}

fn flatten_tuple<T>(input: Vec<(T, Node)>) -> Vec<(T, Node)> {
    input.into_iter().map(|(o, n)| (o, n.flatten())).collect()
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
