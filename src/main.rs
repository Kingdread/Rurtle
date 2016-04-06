#![cfg_attr(feature = "linted", feature(plugin))]
#![cfg_attr(feature = "linted", plugin(clippy))]
// We use this to allow different WindowBuilders for TurtleScreen while still
// retaining the default of a window.
#![feature(default_type_parameter_fallback)]

extern crate bit_vec;
#[macro_use]
extern crate glium;
extern crate glium_text;
extern crate image;
extern crate nalgebra as na;

pub mod graphic;
pub mod turtle;
pub mod lex;
pub mod parse;
pub mod environ;
pub mod readline;
pub mod floodfill;

use std::{env, fs, thread, time, process};
use std::error::Error;
use std::io::Read;
use std::sync::mpsc;

const PROMPT: &'static str = "Rurtle> ";

fn main() {
    let mut environ = {
        let screen = graphic::TurtleScreen::new((640, 640), "Rurtle").unwrap_or_else(|err| {
            println!("Error while creating the screen:");
            println!("  {}", err);
            if let Some(cause) = err.cause() {
                println!("    {}", cause);
            }
            process::exit(1);
        });
        let turtle = turtle::Turtle::new(screen);
        environ::Environment::new(turtle)
    };
    for filename in env::args().skip(1) {
        let mut file = fs::File::open(&filename).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        if let Err(e) = environ.eval_source(&source) {
            println!("[error] {}:", filename);
            println!("{}: {}", e.description(), e);
            return
        }
    };
    let (tx, rx) = mpsc::channel();
    // We use the hermes channel to make the "read thread" wait before printing
    // the next prompt and to signal it when the window closed.
    let (hermes_out, hermes_in) = mpsc::channel();

    // Thread to do the blocking read so we can keep updating the window in the
    // main thread
    let guard = thread::spawn(move || {
        loop {
            let input = readline::readline(PROMPT);
            match input {
                Some(string) => tx.send(string).unwrap(),
                None => break,
            }
            match hermes_in.recv() {
                Ok(false) => (),
                // Ok(true) means the window closed and we should exit
                // Err(..) means the main thread is dead and we should exit
                _ => break,
            };
        }
    });

    loop {
        use std::sync::mpsc::TryRecvError::*;
        let mut send_signal = false;
        let source = match rx.try_recv() {
            Ok(source) => {
                send_signal = true;
                source
            },
            Err(Empty) => "".to_owned(),
            Err(Disconnected) => break,
        };
        if !source.is_empty() {
            readline::add_history(&source);
        }
        if let Err(e) = environ.eval_source(&source) {
            println!("{}: {}", e.description(), e);
        }
        let screen = environ.get_turtle().get_screen();
        screen.draw_and_update();
        screen.handle_events();
        if screen.is_closed() {
            println!("\n\nWindow closed, press enter to exit...");
            break;
        }
        if send_signal {
            hermes_out.send(false).unwrap();
        }
        thread::sleep(time::Duration::from_millis(1000 / 15));
    };
    // We don't really care about the result since the end_signal may already be
    // dropped (e.g. if we got EOF'd). The signal is then unnecessary and the
    // second thread is already dead. We just want the compiler to shut up about
    // "unused result which must be used" :)
    hermes_out.send(true).unwrap_or(());
    guard.join().unwrap();
}
