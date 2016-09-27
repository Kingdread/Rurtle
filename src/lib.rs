//! rurtle provides turtle-graphics in Rust
//!
//! Turtle graphics are a way of doing vector graphics by having a virtual
//! "turtle" on screen. This turtle can be given commands (such as "forward" or
//! "right"), which the turtle will execute while drawing its path on the
//! canvas. For more information see [the Wikipedia
//! entry](https://en.wikipedia.org/wiki/Turtle_graphics).
//!
//! Rurtle can be used in two ways:
//!
//! # Usage as a library ("crate")
//!
//! If you want to embed a turtle in your own program, rurtle exposes the
//! `rurtle::Turtle` and `rurtle::TurtleScreen` structs, which might be of
//! interest. The documentation for the modules `rurtle::turtle` and
//! `rurtle::graphic` provide examples on how to use rurtle in your code.
//!
//! Note that `Turtle`, `TurtleScreen`, `Window` and `Headless` (from
//! `rurtle::graphic::builder`) are reexported.
//!
//! ```no_run
//! use rurtle::{Turtle, TurtleScreen};
//! let screen = TurtleScreen::new((640, 480), "My Turtle").unwrap();
//! let mut turtle = Turtle::new(screen);
//! turtle.forward(100.0);
//! ```
//!
//! Note that you can have multiple turtle screens, and even multiple turtles
//! per screen. However, once created, turtles can't be moved across screens.
//!
//! By default rurtle still compiles all modules, even the ones that are only
//! used to lex and parse rurtle source. If you do not need these functions,
//! you can disable the "cli" feature via cargo, which is otherwise enabled by
//! default. Disabling the "cli" feature disables the `lex`, `parse`, `environ`
//! and `readline` modules, leaving only `turtle`, `graphic` and `floodfill`
//! enabled.
//!
//! # Usage as a standalone application
//!
//! Rurtle comes with a command line interpreter that supports the rurtle
//! language. A description of the language can be found in the `quickstart/`
//! directory in the rurtle source directory. A rendered version can be found on
//! [github](https://github.com/Kingdread/Rurtle/blob/master/quickstart/quickstart.md).
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

#[cfg(feature = "cli")]
pub mod lex;
#[cfg(feature = "cli")]
pub use lex::tokenize;

#[cfg(feature = "cli")]
pub mod parse;
#[cfg(feature = "cli")]
pub use parse::Parser;

#[cfg(feature = "cli")]
pub mod environ;
#[cfg(feature = "cli")]
pub use environ::Environment;

pub mod floodfill;
