//! Abstract Syntax Tree
//!
//! The `ast` module defines types to represent and work with the abstract
//! syntax tree of the Rurtle language. An AST is produced by parsing some token
//! stream.
//!
//! The ast itself does not provide any methods to run or evaluate the program,
//! it is merely a method to represent the program in memory. See the
//! `environ::Environment::eval` method to find a way to execute the program.
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
    /// A block that ignores errors, the first element is the "ordinary" block,
    /// the second element is the block that will be called when an exception
    /// occurs
    TryStatement(Box<Node>, Box<Node>),
    Comparison(Box<Node>, CompOp, Box<Node>),
    /// Addition or subtraction. One addition may hold more than one operation.
    Addition(Box<Node>, Vec<(AddOp, Node)>),
    /// Multiplication and division. One multiplication may hole more than one
    /// operation.
    Multiplication(Box<Node>, Vec<(MulOp, Node)>),
    /// A function call (function, arguments)
    FuncCall(String, Vec<Node>),
    ReturnStatement(Box<Node>),
    Assignment(String, Box<Node>),
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
            TryStatement(normal, exception) => TryStatement(Box::new(normal.flatten()),
                                                            Box::new(exception.flatten())),
            Comparison(operand1, op, operand2) => Comparison(Box::new(operand1.flatten()),
                                                             op,
                                                             Box::new(operand2.flatten())),
            ReturnStatement(value) => ReturnStatement(Box::new(value.flatten())),
            FuncCall(name, args) => FuncCall(name, flatten(args)),
            Assignment(name, value) => Assignment(name, Box::new(value.flatten())),
            node => node,
        }
    }
}

/// Different comparison operators
#[derive(Debug, Copy, Clone)]
pub enum CompOp {
    Equal, Less, Greater, LessEqual, GreaterEqual, NotEqual,
}

macro_rules! okay {
    ($s:expr => $($p:pat), *) => {
        {
            match *$s {
                $($p => true,)*
                _ => false,
            }
        }
    }
}

impl CompOp {
    /// Returns true if CompOp self includes the given Ordering, that is the
    /// CompOp should return true if it compares two elements with the given
    /// Ordering
    ///
    /// ```
    /// # use rurtle::parse::ast::CompOp;
    /// use std::cmp::Ordering;
    /// assert_eq!(CompOp::Equal.matches(&Ordering::Equal), true);
    /// assert_eq!(CompOp::Greater.matches(&Ordering::Less), false);
    /// ```
    pub fn matches(&self, ord: &::std::cmp::Ordering) -> bool {
        use std::cmp::Ordering;
        use self::CompOp::*;
        match *ord {
            Ordering::Less => okay!(self => Less, LessEqual, NotEqual),
            Ordering::Equal => okay!(self => Equal, LessEqual, GreaterEqual),
            Ordering::Greater => okay!(self => Greater, GreaterEqual, NotEqual),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AddOp { Add, Sub }
#[derive(Debug, Copy, Clone)]
pub enum MulOp { Mul, Div }
