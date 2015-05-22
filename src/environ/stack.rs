use super::value::Value;
use std::collections::HashMap;
use std::default::Default;

/// A `Frame` contains information about the current function.
///
/// A new `Frame` is constructed each time you enter a function
#[derive(Debug, Clone)]
pub struct Frame {
    /// Local variables for the function
    pub locals: HashMap<String, Value>,
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
        fn_name: "<global>".to_string(),
        is_global: true,
        .. Frame::default()
    }]
}
