//! The Rurtle type/value system
//!
//! Rurtle is dynamically typed and has 4 different types of values:
//!
//! `Number`: Rurtle doesn't differentiate between integers and floats, there is
//! just a single number type. Internally numbers are represented by floats.
//!
//! `String`: A chain of characters, also known as a text. Like Python, Rurtle
//! has no special datatype for a single character. A `String` of length 1 may
//! thus considered as a character.
//!
//! `List`: A list is a list of other Rurtle Values, possibly even nested
//! Lists. A list is heterogenous, which means that it may contain values of
//! different types.
//!
//! `Nothing`: Something like Python's `None`, this is the default value for
//! everything that doesn't explicitely return something else.
use std::ops;
/// Enum combining the possible Rurtle value types
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Nothing,
    Number(f32),
    String(String),
    List(Vec<Value>),
}

impl Value {
    /// Return the given `Value`'s boolean value. Objects considered `true` are
    ///
    /// * `Numbers` different from 0
    /// * nonempty `String`s and `List`s
    ///
    /// Everything else is considered to be "falsy"
    pub fn boolean(&self) -> bool {
        match *self {
            Value::Number(f) => f != 0.0,
            Value::String(ref s) => !s.is_empty(),
            Value::List(ref l) => !l.is_empty(),
            _ => false,
        }
    }

    /// Return the stringified type of the value
    pub fn type_string(&self) -> &'static str {
        match *self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Nothing => "nothing",
        }
    }
}

impl<'a> ops::Add for &'a Value {
    type Output = Option<Value>;
    /// Add two values together. Not every pair of values may be
    /// combined. Currently the following operations are supported:
    ///
    /// * Number + Number = Number: normal addition
    /// * String + String = String: string concatenation
    /// * String + Number = String: append stringified Number to String
    /// * List + List = List: list concatenation
    /// * List + Other = List: append to list
    ///
    /// May return None if the types can not be added.
    fn add(self, other: &Value) -> Option<Value> {
        match *self {
            Value::Number(a) => {
                match *other {
                    Value::Number(b) => Some(Value::Number(a + b)),
                    _ => None,
                }
            },

            Value::String(ref a) => {
                match *other {
                    Value::String(ref b) => Some(Value::String(format!("{}{}", a, b))),
                    Value::Number(b) => Some(Value::String(format!("{}{}", a, b))),
                    _ => None,
                }
            },

            Value::List(ref a) => {
                match *other {
                    Value::List(ref b) => Some(Value::List(a.iter().chain(b).cloned().collect())),
                    ref b => Some(Value::List({
                        let mut temp = a.clone();
                        temp.push(b.clone());
                        temp
                    })),
                }
            }

            _ => None,
        }
    }
}

impl<'a> ops::Sub for &'a Value {
    type Output = Option<Value>;
    /// Subtracts the second value from the first. This is currently only
    /// meaningful for a `Number` pair. Every other combination will return
    /// `None`
    fn sub(self, other: &Value) -> Option<Value> {
        match *self {
            Value::Number(a) => {
                match *other {
                    Value::Number(b) => Some(Value::Number(a - b)),
                    _ => None,
                }
            },

            _ => None,
        }
    }
}

impl<'a> ops::Mul for &'a Value {
    type Output = Option<Value>;
    /// Multiply two `Values` together. Multiplication is currently defined for
    ///
    /// * Number * Number = Number: normal multiplication
    /// * String * Number = String: replicate the String n times
    /// * List * Number = List: replicate the List n times
    fn mul(self, other: &Value) -> Option<Value> {
        match *self {
            Value::Number(a) => {
                match *other {
                    Value::Number(b) => Some(Value::Number(a * b)),
                    _ => None,
                }
            },

            Value::String(ref a) => {
                match *other {
                    Value::Number(b) => Some(Value::String({
                        let mut temp = String::new();
                        for _ in (0..b as i32) {
                            temp.push_str(a);
                        }
                        temp
                    })),
                    _ => None,
                }
            },

            Value::List(ref a) => {
                match *other {
                    Value::Number(b) => Some(Value::List({
                        let mut temp = Vec::new();
                        for _ in (0..b as i32) {
                            for elem in a.iter() {
                                temp.push(elem.clone());
                            }
                        }
                        temp
                    })),
                    _ => None,
                }
            },

            _ => None,
        }
    }
}

impl<'a> ops::Div for &'a Value {
    type Output = Option<Value>;
    /// Divide one value by another value. Only defined for a pair of `Number`s
    fn div(self, other: &Value) -> Option<Value> {
        match *self {
            Value::Number(a) => {
                match *other {
                    Value::Number(b) => Some(Value::Number(a / b)),
                    _ => None,
                }
            },

            _ => None,
        }
    }
}
