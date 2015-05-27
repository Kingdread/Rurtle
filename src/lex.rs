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

/// Errors that may arise when lexing the input
#[derive(Debug)]
pub enum LexError {
    /// Unterminated string/closing quotes missing
    UnterminatedString,
    /// Invalid number literal
    InvalidNumber(String),
    UnexpectedCharacter(char),
}
impl ::std::fmt::Display for LexError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match *self {
            LexError::UnterminatedString => fmt.pad("unterminated string"),
            LexError::InvalidNumber(ref s) => {
                let s = format!("invalid number: {}", s);
                fmt.pad(&s)
            },
            LexError::UnexpectedCharacter(which) => {
                try!(fmt.pad("unexpected character: "));
                fmt.pad(&which.to_string())
            },
        }
    }
}
impl ::std::error::Error for LexError {
    fn description(&self) -> &str {
        match *self {
            LexError::UnterminatedString => "closing quotes are missing",
            LexError::InvalidNumber(_) => "invalid number literal",
            LexError::UnexpectedCharacter(_) => "unexpected character",
        }
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_identifier_cont(c: char) -> bool {
    is_identifier_start(c) || c.is_alphanumeric()
}

/// Split the input String into single tokens. Strings in the input source are
/// returned as a single token.
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut result = Vec::new();
    let mut chars: Vec<char> = input.chars().collect();
    while !chars.is_empty() {
        let c = chars.remove(0);
        match c {
            '(' => result.push(Token::LParens),
            ')' => result.push(Token::RParens),
            '[' => result.push(Token::LBracket),
            ']' => result.push(Token::RBracket),
            ':' => result.push(Token::Colon),
            '+' => result.push(Token::OpPlus),
            '-' => result.push(Token::OpMinus),
            '*' => result.push(Token::OpMul),
            '/' => result.push(Token::OpDiv),
            '=' => result.push(Token::OpEq),
            '<' => {
                if !chars.is_empty() && chars[0] == '=' {
                    chars.remove(0);
                    result.push(Token::OpLe);
                } else if !chars.is_empty() && chars[0] == '>' {
                    chars.remove(0);
                    result.push(Token::OpNe);
                } else {
                    result.push(Token::OpLt);
                }
            },
            '>' => {
                if !chars.is_empty() && chars[0] == '=' {
                    chars.remove(0);
                    result.push(Token::OpGe);
                } else {
                    result.push(Token::OpGt);
                }
            },
            // Ignore comments, i.e. everything from ; to the end of line
            ';' => {
                while !chars.is_empty() && chars.remove(0) != '\n' {}
            },
            // Parse an identifier or a keyword
            _ if is_identifier_start(c) => {
                let mut word = c.to_string();
                while !chars.is_empty() && is_identifier_cont(chars[0]) {
                    word.push(chars.remove(0));
                }
                result.push(match word.to_uppercase().as_ref() {
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
                    Ok(f) => result.push(Token::Number(f)),
                    Err(_) => return Err(LexError::InvalidNumber(number)),
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
                            result.push(Token::String(string));
                            terminated = true;
                            break;
                        },
                        _ => string.push(c),
                    }
                }
                if !terminated {
                    return Err(LexError::UnterminatedString);
                }
            },
            _ if c.is_whitespace() => {},
            _ => return Err(LexError::UnexpectedCharacter(c)),
        }
    }
    Ok(result)
}
