//! Parsing module for Rurtle programs.
//!
//! Note that parsing requires some additional information, i.e. the number of
//! arguments for a function. Function calls in Rurtle need neither parenthesis
//! nor something else, so this is legal:
//!
//! ```text
//! FUNCA FUNCB 10
//! ```
//!
//! Depending on how many arguments each function takes, this may be parsed as
//! either `funca(funcb(10))` or `funca(funcb(), 10)`.
//!
//! # Grammar
//!
//! A EBNF-like (incomplete) grammar may look like
//!
//! ```text
//! root := {statement} ;
//! statement := learn-def | if-stmt | repeat-stmt | while-stmt | return-stmt |
//!              try-stmt | expression ;
//! learn-def := 'LEARN' identifier {variable} 'DO' {statement} 'END' ;
//! if-stmt := 'IF' expression 'DO' {statement} ['ELSE' {statement}]'END' ;
//! repeat-stmt := 'REPEAT' expression 'DO' {statement} 'END' ;
//! while-stmt := 'WHILE' expression 'DO' {statement} 'END' ;
//! return-stmt := 'RETURN' expression ;
//! try-stmt := 'TRY' {statement} 'ELSE' {statement} 'END' ;
//! variable := ':' identifier ;
//! identifier := idenfitier-start {identifier-cont} ;
//! idenfitier-start := <any alphabetic character> ;
//! idenfitier-cont := <any alpabetic or numeric character> ;
//! expression := comparison ;
//! comparison := expr [comp_op expr] ;
//! comp_op := '=' | '<' | '>' | ''<=' | '>=' | '<>' ;
//! expr := product {('+' | '-') product} ;
//! product := factor {('*' | '/') factor} ;
//! factor := '(' expression ')' | list | variable | string | number | (identifier {expression}) ;
//! list := '[' {expression} ']' ;
//! string := '"' {<any character>} '"' ;
//! number := ['+' | '-'] <any valid floating point number literal> ;
//! ```
pub mod ast;

use super::lex::{Token, MetaToken};
use self::ast::{Node, AddOp, MulOp, CompOp};
use self::ast::Node::*;
use std::collections::{HashMap, VecDeque};
use std::{error, fmt};

/// A `FuncMap` maps the name of a function to the number of arguments it takes
pub type FuncMap = HashMap<String, i32>;

/// A `Parser` builds an AST from the given input token stream.
pub struct Parser {
    tokens: VecDeque<MetaToken>,
    scope_stack: Vec<Scope>,
    last_line: u32,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    UnexpectedToken(&'static str, Token),
    UnexpectedEnd,
    UnknownFunction(String),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::ParseErrorKind::*;
        match *self {
            UnexpectedToken(expected, ref got) => {
                try!(fmt.pad("unexpected token, expected '"));
                try!(fmt.pad(expected));
                try!(fmt.pad("', got '"));
                try!(got.fmt(fmt));
                fmt.pad("'")
            },
            UnexpectedEnd => fmt.pad("unexpected end"),
            UnknownFunction(ref name) => {
                try!(fmt.pad("unknown function: "));
                name.fmt(fmt)
            }
        }
    }
}

// Error returns are pretty long anyway
use self::ParseErrorKind::*;

#[derive(Debug)]
pub struct ParseError {
    line_number: u32,
    kind: ParseErrorKind,
}
impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let text = format!("Error in line {}: {}", self.line_number, self.kind);
        fmt.pad(&text)
    }
}
impl error::Error for ParseError {
    fn description(&self) -> &str {
        match self.kind {
            UnexpectedToken(..) => "unexpected token",
            UnexpectedEnd => "unexpected end",
            UnknownFunction(..) => "unknown function",
        }
    }
}

pub type ParseResult = Result<Node, ParseError>;

#[derive(Debug, Clone)]
struct Scope {
    functions: FuncMap,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            functions: FuncMap::new(),
        }
    }
}

/// Always returns an `Err` value but attaches the required meta information
/// (such as line number)
macro_rules! parse_error {
    ($s:expr, $k:expr) => {
        {
            // This is a very dirty hack to make clippy shut up about "needless return"
            // we can't just omit return here since the macro may be used to exit a
            // function early.
            // The "if true" should be optmized away, but it's enough to make rustc and
            // clippy happy. And if they're happy, I am too.
            if true {
                return Err(ParseError {
                    line_number: $s.last_line,
                    kind: $k,
                })
            };
            unreachable!("parse_error goofed, true no longer considered true")
        }
    }
}

macro_rules! expect {
    ($s:expr, $t:path) => {
        {
            let token = try!($s.pop_left());
            match token {
                $t => (),
                _ => parse_error!($s, UnexpectedToken(stringify!($t), token)),
            }
        }
    }
}

impl Parser {
    /// Construct a new `Parser`, consuming the given tokens.
    pub fn new(tokens: VecDeque<MetaToken>, functions: FuncMap) -> Parser {
        let global_scope = Scope {
            functions: functions,
        };
        Parser {
            tokens: tokens,
            scope_stack: vec![global_scope],
            last_line: 0,
        }
    }

    /// Attempt to return the root node
    pub fn parse(&mut self) -> ParseResult {
        self.parse_statement_list()
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scope_stack.last_mut().expect("scope_stack is empty, should have global scope")
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(Scope::new())
    }

    fn pop_scope(&mut self) {
        debug_assert!(self.scope_stack.len() > 1, "Trying to pop global scope");
        self.scope_stack.pop().expect("scope_stack is empty, should have global scope");
    }

    fn find_function_arg_count(&self, name: &str) -> Option<i32> {
        for scope in self.scope_stack.iter().rev() {
            let function_map = &scope.functions;
            match function_map.get(name) {
                Some(i) => return Some(*i),
                None => {},
            }
        }
        None
    }

    fn peek(&self) -> Token {
        self.tokens.front().unwrap().token.clone()
    }

    fn pop_left(&mut self) -> Result<Token, ParseError> {
        if let Some(meta) = self.tokens.pop_front() {
            self.last_line = meta.line_number;
            Ok(meta.token)
        } else {
            parse_error!(self, UnexpectedEnd)
        }
    }

    fn parse_statement_list(&mut self) -> ParseResult {
        let mut statements = Vec::new();
        while !self.tokens.is_empty() {
            let statement = try!(self.parse_statement());
            statements.push(statement);
        }
        Ok(StatementList(statements))
    }

    fn parse_loop_body(&mut self) -> ParseResult {
        // Loop bodies generally introduce new scopes
        self.push_scope();
        let mut statements = Vec::new();
        while !self.tokens.is_empty() {
            match self.peek() {
                Token::KeyElse | Token::KeyEnd => break,
                _ => {
                    statements.push(try!(self.parse_statement()));
                },
            }
        }
        self.pop_scope();
        Ok(StatementList(statements))
    }

    fn parse_statement(&mut self) -> ParseResult {
        let token = self.peek();
        match token {
            Token::KeyLearn => self.parse_learn_stmt(),
            Token::KeyIf => self.parse_if_stmt(),
            Token::KeyRepeat => self.parse_repeat_stmt(),
            Token::KeyWhile => self.parse_while_stmt(),
            Token::KeyReturn => self.parse_return_stmt(),
            Token::KeyTry => self.parse_try_stmt(),
            _ => self.parse_expression(),
        }
    }

    fn parse_learn_stmt(&mut self) -> ParseResult {
        expect!(self, Token::KeyLearn);
        let name = match try!(self.pop_left()) {
            Token::Word(string) => string.to_uppercase(),
            token => parse_error!(self, UnexpectedToken("Token::Word", token)),
        };
        let mut variables = Vec::new();
        while !self.tokens.is_empty() {
            match try!(self.pop_left()) {
                Token::Colon => {
                    match try!(self.pop_left()) {
                        Token::Word(s) => variables.push(s),
                        token => parse_error!(self, UnexpectedToken("Token::Word", token)),
                    }
                },
                Token::KeyDo => break,
                token => parse_error!(self, UnexpectedToken("Token::KeyDo", token)),
            }
        }
        // We need the argument count for this function if it appears later
        // during the parsing stage (e.g. in a recursive call)
        self.current_scope_mut().functions.insert(name.clone(), variables.len() as i32);
        let statements = try!(self.parse_loop_body());
        expect!(self, Token::KeyEnd);
        Ok(LearnStatement(name, variables, Box::new(statements)))
    }

    fn parse_if_stmt(&mut self) -> ParseResult {
        expect!(self, Token::KeyIf);
        let condition = Box::new(try!(self.parse_expression()));
        expect!(self, Token::KeyDo);
        let true_body = Box::new(try!(self.parse_loop_body()));
        let false_body = if let Token::KeyElse = self.peek() {
            try!(self.pop_left());
            Some(Box::new(try!(self.parse_loop_body())))
        } else { None };
        expect!(self, Token::KeyEnd);
        Ok(IfStatement(condition, true_body, false_body))
    }

    fn parse_repeat_stmt(&mut self) -> ParseResult {
        expect!(self, Token::KeyRepeat);
        let number = Box::new(try!(self.parse_expression()));
        expect!(self, Token::KeyDo);
        let body = try!(self.parse_loop_body());
        expect!(self, Token::KeyEnd);
        Ok(RepeatStatement(number, Box::new(body)))
    }

    fn parse_while_stmt(&mut self) -> ParseResult {
        expect!(self, Token::KeyWhile);
        let condition = Box::new(try!(self.parse_expression()));
        expect!(self, Token::KeyDo);
        let body = try!(self.parse_loop_body());
        expect!(self, Token::KeyEnd);
        Ok(WhileStatement(condition, Box::new(body)))
    }

    fn parse_return_stmt(&mut self) -> ParseResult {
        expect!(self, Token::KeyReturn);
        let result = Box::new(try!(self.parse_expression()));
        Ok(ReturnStatement(result))
    }

    fn parse_try_stmt(&mut self) -> ParseResult {
        expect!(self, Token::KeyTry);
        let normal = Box::new(try!(self.parse_loop_body()));
        expect!(self, Token::KeyElse);
        let exception = Box::new(try!(self.parse_loop_body()));
        expect!(self, Token::KeyEnd);
        Ok(TryStatement(normal, exception))
    }

    fn parse_expression(&mut self) -> ParseResult {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> ParseResult {
        let operand = try!(self.parse_expr());
        if self.tokens.is_empty() {
            return Ok(operand);
        };
        match self.peek() {
            Token::OpEq | Token::OpLt | Token::OpGt |
            Token::OpLe | Token::OpGe | Token::OpNe => {
                let op = match try!(self.pop_left()) {
                    Token::OpEq => CompOp::Equal,
                    Token::OpLt => CompOp::Less,
                    Token::OpGt => CompOp::Greater,
                    Token::OpLe => CompOp::LessEqual,
                    Token::OpGe => CompOp::GreaterEqual,
                    Token::OpNe => CompOp::NotEqual,
                    _ => unreachable!(),
                };
                let operand_right = Box::new(try!(self.parse_expr()));
                Ok(Comparison(Box::new(operand), op, operand_right))
            }
            _ => Ok(operand),
        }
    }

    fn parse_expr(&mut self) -> ParseResult {
        let product = Box::new(try!(self.parse_product()));
        let mut addends = Vec::new();
        while !self.tokens.is_empty() {
            match self.peek() {
                Token::OpPlus | Token::OpMinus => {
                    let op = match try!(self.pop_left()) {
                        Token::OpPlus => AddOp::Add,
                        Token::OpMinus => AddOp::Sub,
                        _ => unreachable!(),
                    };
                    addends.push((op, try!(self.parse_product())));
                },
                _ => break,
            }
        }
        Ok(Addition(product, addends))
    }

    fn parse_product(&mut self) -> ParseResult {
        let factor = Box::new(try!(self.parse_factor()));
        let mut factors = Vec::new();
        while !self.tokens.is_empty() {
            match self.peek() {
                Token::OpMul | Token::OpDiv => {
                    let op = match try!(self.pop_left()) {
                        Token::OpMul => MulOp::Mul,
                        Token::OpDiv => MulOp::Div,
                        _ => unreachable!(),
                    };
                    factors.push((op, try!(self.parse_factor())));
                },
                _ => break,
            }
        }
        Ok(Multiplication(factor, factors))
    }

    fn parse_factor(&mut self) -> ParseResult {
        if self.tokens.is_empty() {
            parse_error!(self, UnexpectedEnd);
        };
        match try!(self.pop_left()) {
            Token::LParens => {
                let factor = try!(self.parse_expression());
                expect!(self, Token::RParens);
                Ok(factor)
            },
            Token::LBracket => {
                let mut list = Vec::new();
                while !self.tokens.is_empty() {
                    if let Token::RBracket = self.peek() {
                        break
                    }
                    list.push(try!(self.parse_expression()));
                }
                expect!(self, Token::RBracket);
                Ok(List(list))
            },
            Token::Colon => {
                if let Token::Word(name) = try!(self.pop_left()) {
                    if self.tokens.is_empty() {
                        Ok(Variable(name))
                    } else {
                        if let Token::OpDefine = self.peek() {
                            try!(self.pop_left());
                            let value = try!(self.parse_expression());
                            Ok(Assignment(name, Box::new(value)))
                        } else {
                            Ok(Variable(name))
                        }
                    }
                } else {
                    parse_error!(self, UnexpectedToken("Token::Word", try!(self.pop_left())))
                }
            },
            // A function call
            Token::Word(name) => {
                let argument_count = match self.find_function_arg_count(&name.to_uppercase()) {
                    Some(i) => i,
                    None => parse_error!(self, UnknownFunction(name)),
                };
                let mut arguments = Vec::new();
                for _ in 0..argument_count {
                    arguments.push(try!(self.parse_expression()));
                }
                Ok(FuncCall(name, arguments))
            },
            Token::String(string) => Ok(StringLiteral(string)),
            Token::Number(num) => Ok(Number(num)),
            // Unary prefixes for numbers
            Token::OpMinus => {
                match try!(self.pop_left()) {
                    Token::Number(num) => Ok(Number(-num)),
                    token => parse_error!(self, UnexpectedToken("Token::Number", token)),
                }
            },
            Token::OpPlus => {
                match try!(self.pop_left()) {
                    Token::Number(num) => Ok(Number(num)),
                    token => parse_error!(self, UnexpectedToken("Token::Number", token)),
                }
            },
            token => parse_error!(self, UnexpectedToken("expression", token)),
        }
    }
}
