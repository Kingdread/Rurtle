//! rurtle provides turtle-graphics in Rust
//!
//! # Windows support
//!
//! rurtle should run fine under Windows, it uses no Linux-specific functions.
//! However, since the addition of text support (a0b9483), we link to freetype.
//! To install freetype on Windows please see [the Piston getting started guide]
//! (https://github.com/bvssvni/Piston-Tutorials/tree/master/getting-started#freetype-on-windows).
#![cfg_attr(feature = "linted", feature(plugin))]
#![cfg_attr(feature = "linted", plugin(clippy))]

extern crate bit_vec;
#[macro_use]
extern crate glium;
extern crate glium_text;
extern crate image;
extern crate nalgebra as na;

pub mod graphic;
pub use graphic::TurtleScreen;
pub use graphic::color;
pub use graphic::builder::{Window, Headless};

pub mod turtle;
pub use turtle::Turtle;

pub mod lex;
pub use lex::tokenize;

pub mod parse;
pub use parse::Parser;

pub mod environ;
pub use environ::Environment;

pub mod readline;

pub mod floodfill;
