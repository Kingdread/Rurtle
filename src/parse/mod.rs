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
//! statement := learn-def | if-stmt | repeat-stmt | while-stmt | return-stmt | expression ;
//! learn-def := 'LEARN' identifier {variable} 'DO' {statement} 'END' ;
//! if-stmt := 'IF' expression 'DO' {statement} ['ELSE' {statement}]'END' ;
//! repeat-stmt := 'REPEAT' expression 'DO' {statement} 'END' ;
//! while-stmt := 'WHILE' expression 'DO' {statement} 'END' ;
//! return-stmt := 'RETURN' expression ;
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
//! number := <any valid floating point number literal> ;
//! ```
pub mod ast;

use super::lex::Token;
use self::ast::{Node, AddOp, MulOp, CompOp};
use self::ast::Node::*;
use std::collections::HashMap;

/// A `FuncMap` maps the name of a function to the number of arguments it takes
pub type FuncMap = HashMap<String, i32>;

/// A `Parser` builds an AST from the given input token stream.
pub struct Parser {
    tokens: Vec<Token>,
    functions: FuncMap,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(&'static str, Token),
    UnexpectedEnd,
    UnknownFunction(String),
}

impl ::std::fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        use self::ParseError::*;
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

impl ::std::error::Error for ParseError {
    fn description(&self) -> &str {
        use self::ParseError::*;
        match *self {
            UnexpectedToken(..) => "unexpected token",
            UnexpectedEnd => "unexpected end of input",
            UnknownFunction(_) => "the parser doesn't know the function",
        }
    }
}

pub type ParseResult = Result<Node, ParseError>;

macro_rules! expect {
    ($s:expr, $t:path) => {
        {
            if $s.tokens.is_empty() {
                return Err(ParseError::UnexpectedEnd);
            };
            let token = $s.pop_left();
            match token {
                $t(..) => (),
                _ => return Err(ParseError::UnexpectedToken(stringify!($t), token)),
            }
        }
    }
}

macro_rules! pop_left {
    ($s:expr) => {
        {
            if $s.tokens.is_empty() {
                return Err(ParseError::UnexpectedEnd)
            } else {
                $s.pop_left()
            }
        }
    }
}

impl Parser {
    /// Construct a new `Parser`, consuming the given tokens.
    pub fn new(tokens: Vec<Token>, functions: FuncMap) -> Parser {
        Parser {
            tokens: tokens,
            functions: functions,
        }
    }

    /// Attempt to return the root node
    pub fn parse(&mut self) -> ParseResult {
        self.parse_statement_list()
    }

    fn peek(&self) -> Token {
        self.tokens[0].clone()
    }

    fn pop_left(&mut self) -> Token {
        self.tokens.remove(0)
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
        let mut statements = Vec::new();
        while !self.tokens.is_empty() {
            match self.peek() {
                Token::KeyElse | Token::KeyEnd => break,
                _ => {
                    statements.push(try!(self.parse_statement()));
                },
            }
        }
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
            _ => self.parse_expression(),
        }
    }

    fn parse_learn_stmt(&mut self) -> ParseResult {
        expect!(self, Token::KeyLearn);
        let name = match pop_left!(self) {
            Token::Word(string) => string.to_uppercase(),
            token => return Err(ParseError::UnexpectedToken("Token::Word", token)),
        };
        let mut variables = Vec::new();
        while !self.tokens.is_empty() {
            match pop_left!(self) {
                Token::Colon => {
                    match pop_left!(self) {
                        Token::Word(s) => variables.push(s),
                        token => return Err(ParseError::UnexpectedToken("Token::Word", token)),
                    }
                },
                Token::KeyDo => break,
                token => return Err(ParseError::UnexpectedToken("Token::KeyDo", token)),
            }
        }
        // We need the argument count for this function if it appears later
        // during the parsing stage (e.g. in a recursive call)
        self.functions.insert(name.clone(), variables.len() as i32);
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
            self.pop_left();
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
                let op = match self.pop_left() {
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
                    let op = match self.pop_left() {
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
                    let op = match self.pop_left() {
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
            return Err(ParseError::UnexpectedEnd);
        };
        match self.pop_left() {
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
                if let Token::Word(name) = self.peek() {
                    self.pop_left();
                    Ok(Variable(name))
                } else {
                    Err(ParseError::UnexpectedToken("Token::Word", self.pop_left()))
                }
            },
            // A function call
            Token::Word(name) => {
                let argument_count = match self.functions.get(&name.to_uppercase()) {
                    Some(i) => *i,
                    None => return Err(ParseError::UnknownFunction(name)),
                };
                let mut arguments = Vec::new();
                for _ in (0..argument_count) {
                    arguments.push(try!(self.parse_expression()));
                }
                Ok(FuncCall(name, arguments))
            },
            Token::String(string) => Ok(StringLiteral(string)),
            Token::Number(num) => Ok(Number(num)),
            token => Err(ParseError::UnexpectedToken("expression", token)),
        }
    }
}
