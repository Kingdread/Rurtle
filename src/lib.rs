//! rurtle provides turtle-graphics in Rust
//!
//! # Windows support
//!
//! rurtle should run fine under Windows, it uses no Linux-specific functions.
//! However, since the addition of text support (a0b9483), we link to freetype.
//! This means that you need libfreetype-6.dll, which you can download from
//! [this](https://github.com/Kingdread/libfreetype-dll) repository or find with
//! a quick Google search.
//!
//! Once you have the libfreetype-6.dll for your platform, you need to copy it
//! to two places:
//!
//! Rust needs to find the DLL during compilation, which is why you should copy
//! it to `<Rust install dir>\bin\rustlib\<target triple>\lib\`.
//!
//! Additionally, Windows needs to find the file when you run the program, which
//! is why you also need to copy it to one of the following places:
//!
//! * The same folder as the executable
//! * The folder you run the program from
//! * `C:\Windows\System32`
//! * `C:\Windows`
//! * One of the directories in `%PATH%`
//!
//! For more information see [Search Path Used by Windows to Locate a DLL]
//! (https://msdn.microsoft.com/en-us/library/7d83bc18.aspx)
#[macro_use]
extern crate glium;

pub mod graphic;
pub use graphic::TurtleScreen;
pub use graphic::color;

pub mod turtle;
pub use turtle::Turtle;

pub mod lex;
pub use lex::tokenize;

pub mod parse;
pub use parse::Parser;

pub mod environ;
pub use environ::Environment;

pub mod readline;
