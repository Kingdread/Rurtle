//! Core readline functionality
//!
//! This module is just a wrapper for a small FFI interface to the C readline
//! library.
//!
//! This module is tailored to the needs of the Rurtle CLI and not intended as a
//! general readline-rust wrapper.
//!
//! # Windows support
//!
//! Note that readline is not available on Windows. `readline` will still work
//! with less awesomeness though, and `add_history` is just a stub.
//!
//! This module will automatically select the right readline version for the
//! target os.
#[cfg(not(windows))]
mod module {
    extern crate libc;
    use self::libc::{c_void, free};
    use std::ffi::{CString, CStr};

    mod sys {
        use super::libc::{c_char};
        #[link(name = "readline")]
        extern {
            pub fn readline(prompt: *const c_char) -> *mut c_char;
            pub fn add_history(line: *const c_char);
        }
    }

    /// Display the given prompt and return the input line. If readline encounters
    /// an EOF, `None` is returned.
    ///
    /// The input data is treated as valid UTF-8 data. Errors during the conversion
    /// are silently swallowed.
    ///
    /// The underlying `malloc()`'d buffer that `readline()` returns is `free()`'d
    /// and the converted owned `String` is returned.
    ///
    /// # Panics
    ///
    /// This function panics if the given prompt contains nul-bytes ('\0')
    pub fn readline(prompt: &str) -> Option<String> {
        let c_prompt = CString::new(prompt.to_string()).ok()
            .expect("The given prompt contains NUL bytes");
        let result_ptr = unsafe { sys::readline(c_prompt.as_ptr()) };
        // If readline returns NULL we know that EOF is encountered.
        if result_ptr.is_null() {
            return None
        }
        let result_c_str = unsafe { CStr::from_ptr(result_ptr) };
        let result = String::from_utf8_lossy(result_c_str.to_bytes()).into_owned();
        unsafe {
            free(result_c_str.as_ptr() as *mut c_void);
        };
        Some(result)
    }

    /// Add the given line to the readline history so the user can navigate back to it.
    ///
    /// Note that you should not add empty lines to the history.
    ///
    /// # Panics
    ///
    /// This function panics if the given line contains nul-bytes ('\0')
    pub fn add_history(line: &str) {
        let c_line = CString::new(line.as_bytes()).ok()
            .expect("The given line contains NUL bytes");
        unsafe {
            sys::add_history(c_line.as_ptr());
        }
    }
}

#[cfg(windows)]
mod module {
    use std::io::{self, Write};
    
    pub fn readline(prompt: &str) -> Option<String> {
        print!("{}", prompt);
        io::stdout().flush().unwrap();
        let mut stdin = io::stdin();
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => Some(line),
            Err(e) => panic!(format!("Error in readline: {}", e)),
        }
    }
    
    pub fn add_history(_: &str) {}
}

pub use self::module::*;