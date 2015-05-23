#![feature(scoped)]
#[macro_use]
extern crate glium;

pub mod graphic;
pub mod turtle;
pub mod lex;
pub mod parse;
pub mod environ;

use std::error::Error;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::thread;
use std::sync::mpsc;

fn main() {
    let screen = graphic::TurtleScreen::new((640, 640), "Rurtle");
    let turtle = turtle::Turtle::new(screen);
    let mut environ = environ::Environment::new(turtle);
    for filename in env::args().skip(1) {
        let mut file = fs::File::open(filename).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        if let Err(e) = environ.eval_source(&source) {
            println!("{}: {}", e.description(), e);
            return;
        }
    };
    let (tx, rx) = mpsc::channel();
    let (end_signaler, end_signal) = mpsc::channel();
    let guard = thread::scoped(move || {
        let mut stdin = io::stdin();
        let mut buffer = [0u8; 128];
        loop {
            if let Ok(_) = end_signal.try_recv() {
                break;
            };
            let count = stdin.read(&mut buffer).unwrap();
            if count == 0 { break; }
            for i in (0..count) {
                tx.send(buffer[i]).unwrap();
            }
        }
    });
    'out: loop {
        let mut data = Vec::new();
        loop {
            use std::sync::mpsc::TryRecvError::*;
            match rx.try_recv() {
                Ok(c) => data.push(c),
                Err(Empty) => break,
                Err(Disconnected) => break 'out,
            }
        }
        let source = String::from_utf8_lossy(&data);
        if let Err(e) = environ.eval_source(&source) {
            println!("{}: {}", e.description(), e);
        }
        let screen = environ.get_turtle().get_screen();
        screen.draw_and_update();
        screen.handle_events();
        if screen.is_closed() {
            println!("Window closed, press enter to exit...");
            break;
        }
        thread::sleep_ms(1000 / 15);
    };
    end_signaler.send(()).unwrap();
    guard.join();
}
