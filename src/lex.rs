//! This module contains the lexical analyser for the Turtle language. It can
//! split a Rurtle source string into single tokens, which can be later used by
//! the interpreter.
//!
//! Valid identifiers start with any (unicode) alphabetic character and may
//! consist of any alpha-numeric character thereafter.
//!
//! Strings have to be enclosed in double quotes ("), there are no strings in
//! enclosed in lists. For example, this is valid: "Hello", this is not: [Hello]
//!
//! Lists are enclosed in []-brackets.
//!
//! Variables are prefixed by a colon (:) and otherwise follow the same rules as
//! identifiers.

/// A `Token` represents a "atom" block of the input source.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// An identifier, also called Word
    Word(String),
    Number(f32),
    /// The left bracket [
    LBracket,
    /// The right bracket ]
    RBracket,
    /// The left parenthesis (
    LParens,
    /// The right parenthesis )
    RParens,
    /// The colon :
    Colon,
    /// A String enclosed in "quotes"
    String(String),
    /// Operator "equals" =
    OpEq,
    /// Operator "less than" <
    OpLt,
    /// Operator "greater than" >
    OpGt,
    /// Operator "less or equal" <=
    OpLe,
    /// Operator "greater or equal" >=
    OpGe,
    /// Operator "not equal" <>
    OpNe,
    /// Operator "plus" +
    OpPlus,
    /// Operator "minues" -
    OpMinus,
    /// Operator "multipication" *
    OpMul,
    /// Operator "division" /
    OpDiv,
    /// Keyword "LEARN"
    KeyLearn,
    /// Keyword "DO"
    KeyDo,
    /// Keyword "ELSE"
    KeyElse,
    /// Keyword "REPEAT"
    KeyRepeat,
    /// Keyword "WHILE"
    KeyWhile,
    /// Keyword "IF"
    KeyIf,
    /// Keyword "END"
    KeyEnd,
    /// Keyword "FOR"
    KeyFor,
    /// Keyword "RETURN"
    KeyReturn,
}

impl ::std::fmt::Display for Token {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        use self::Token::*;
        let debug = format!("{:?}", self);
        fmt.pad(match *self {
            Word(_) => "word",
            Number(_) => "number",
            LBracket => "left bracket",
            RBracket => "right bracket",
            LParens => "left parenthesis",
            RParens => "right parenthesis",
            Colon => "colon",
            String(_) => "string literal",
            _ => &debug,
        })
    }
}

/// This struct contains a token and some additional meta information
#[derive(Clone, Debug)]
pub struct MetaToken {
    /// The actual token
    pub token: Token,
    /// Line number in which the token was found. Lines start with 1.
    pub line_number: u32,
}

/// Errors that may arise when lexing the input. The first member is always the line number.
#[derive(Debug)]
pub enum LexError {
    /// Unterminated string/closing quotes missing
    UnterminatedString(u32),
    /// Invalid number literal
    InvalidNumber(u32, String),
    UnexpectedCharacter(u32, char),
}
impl ::std::fmt::Display for LexError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            LexError::UnterminatedString(line) => {
                try!(fmt.pad("unterminated string in line "));
                line.fmt(fmt)
            },
            LexError::InvalidNumber(line, ref s) => {
                let s = format!("invalid number: {} in line {}", s, line);
                fmt.pad(&s)
            },
            LexError::UnexpectedCharacter(line, which) => {
                try!(fmt.pad("unexpected character in line "));
                try!(line.fmt(fmt));
                try!(fmt.pad(": "));
                fmt.pad(&which.to_string())
            },
        }
    }
}
impl ::std::error::Error for LexError {
    fn description(&self) -> &str {
        match *self {
            LexError::UnterminatedString(..) => "closing quotes are missing",
            LexError::InvalidNumber(..) => "invalid number literal",
            LexError::UnexpectedCharacter(..) => "unexpected character",
        }
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_identifier_cont(c: char) -> bool {
    is_identifier_start(c) || c.is_alphanumeric()
}

struct Tokenizer {
    result: Vec<MetaToken>,
    line_number: u32,
}

impl Tokenizer {
    fn new() -> Tokenizer {
        Tokenizer {
            result: Vec::new(),
            line_number: 1,
        }
    }

    fn push(&mut self, token: Token) {
        self.result.push(MetaToken {
            token: token,
            line_number: self.line_number,
        })
    }

    fn tokenize(mut self, input: &str) -> Result<Vec<MetaToken>, LexError> {
        let mut chars: Vec<char> = input.chars().collect();
        while !chars.is_empty() {
            let c = chars.remove(0);
            match c {
                '(' => self.push(Token::LParens),
                ')' => self.push(Token::RParens),
                '[' => self.push(Token::LBracket),
                ']' => self.push(Token::RBracket),
                ':' => self.push(Token::Colon),
                '+' => self.push(Token::OpPlus),
                '-' => self.push(Token::OpMinus),
                '*' => self.push(Token::OpMul),
                '/' => self.push(Token::OpDiv),
                '=' => self.push(Token::OpEq),
                '<' => {
                    if !chars.is_empty() && chars[0] == '=' {
                        chars.remove(0);
                        self.push(Token::OpLe);
                    } else if !chars.is_empty() && chars[0] == '>' {
                        chars.remove(0);
                        self.push(Token::OpNe);
                    } else {
                        self.push(Token::OpLt);
                    }
                },
                '>' => {
                    if !chars.is_empty() && chars[0] == '=' {
                        chars.remove(0);
                        self.push(Token::OpGe);
                    } else {
                        self.push(Token::OpGt);
                    }
                },
                // Ignore comments, i.e. everything from ; to the end of line
                ';' => {
                    while !chars.is_empty() {
                        if chars.remove(0) == '\n' {
                            self.line_number += 1;
                            break
                        }
                    }
                },
                // Parse an identifier or a keyword
                _ if is_identifier_start(c) => {
                    let mut word = c.to_string();
                    while !chars.is_empty() && is_identifier_cont(chars[0]) {
                        word.push(chars.remove(0));
                    }
                    self.push(match word.to_uppercase().as_ref() {
                        "LEARN" => Token::KeyLearn,
                        "DO" => Token::KeyDo,
                        "END" => Token::KeyEnd,
                        "REPEAT" => Token::KeyRepeat,
                        "FOR" => Token::KeyFor,
                        "IF" => Token::KeyIf,
                        "WHILE" => Token::KeyWhile,
                        "RETURN" => Token::KeyReturn,
                        "ELSE" => Token::KeyElse,
                        _ => Token::Word(word),
                    });
                },
                // Parse a number literal
                _ if c.is_numeric() => {
                    let mut number = c.to_string();
                    while !chars.is_empty() &&
                        (chars[0].is_numeric() || chars[0] == '.') {
                        number.push(chars.remove(0));
                    }
                    match number.parse() {
                        Ok(f) => self.push(Token::Number(f)),
                        Err(_) => return Err(LexError::InvalidNumber(self.line_number, number)),
                    }
                },
                // Parse a String literal
                '"' => {
                    let mut string = String::new();
                    let mut terminated = false;
                    while !chars.is_empty() {
                        let c = chars.remove(0);
                        match c {
                            '"' => {
                                self.push(Token::String(string));
                                terminated = true;
                                break;
                            },
                            '\n' => {
                                self.line_number += 1;
                                string.push(c);
                            },
                            _ => string.push(c),
                        }
                    }
                    if !terminated {
                        return Err(LexError::UnterminatedString(self.line_number));
                    }
                },
                '\n' => self.line_number += 1,
                _ if c.is_whitespace() => {},
                _ => return Err(LexError::UnexpectedCharacter(self.line_number, c)),
            }
        }
        Ok(self.result)
    }
}

/// Split the input String into single tokens. Strings in the input source are
/// returned as a single token.
pub fn tokenize(input: &str) -> Result<Vec<MetaToken>, LexError> {
    let tokenizer = Tokenizer::new();
    tokenizer.tokenize(input)
}
