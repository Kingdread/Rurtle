use super::value::Value;
use super::Function;
use super::functions;
use std::collections::HashMap;
use std::default::Default;

/// A `Frame` contains information about the current function.
///
/// A new `Frame` is constructed each time you enter a function
#[derive(Debug, Clone)]
pub struct Frame {
    /// Local variables for the function
    pub locals: HashMap<String, Value>,
    /// Functions contained in this frame
    ///
    /// The scope of function definitions and variables are different, a frame
    /// always refers to a function call so the scope of variables is the current
    /// function. Function definitions however are also restricted to blocks of
    /// if statements, so this is a "stack inside a stack"
    pub functions: Vec<HashMap<String, Function>>,
    /// If this flag is set, the current function should return
    pub should_return: bool,
    /// Value that the current function should return (if any)
    pub return_value: Option<Value>,
    /// Name of the function
    pub fn_name: String,
    /// Flag indicating if this frame is the global frame
    pub is_global: bool,
}

impl Default for Frame {
    fn default() -> Frame {
        Frame {
            locals: HashMap::new(),
            functions: vec![HashMap::new()],
            should_return: false,
            return_value: None,
            fn_name: String::new(),
            is_global: false,
        }
    }
}

/// Return a new stack with the root frame (global frame) constructed
pub fn new_stack() -> Vec<Frame> {
    vec![Frame {
        functions: vec![functions::default_functions()],
        fn_name: "<global>".to_owned(),
        is_global: true,
        .. Frame::default()
    }]
}
