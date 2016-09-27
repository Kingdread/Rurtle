#![cfg_attr(feature = "linted", feature(plugin))]
#![cfg_attr(feature = "linted", plugin(clippy))]
extern crate rurtle;
extern crate image;
extern crate rustyline;
extern crate docopt;
extern crate rustc_serialize;

#[cfg(feature = "cli")]
use rurtle::{graphic, turtle, environ};
#[cfg(feature = "cli")]
use rurtle::environ::value::Value;

use std::{fs, thread, time, process};
use std::error::Error;
use std::io::Read;
use std::sync::{mpsc, Arc, Mutex};

use rustyline::Editor;

use docopt::Docopt;

const PROMPT: &'static str = "Rurtle> ";

const USAGE: &'static str = "
Rurtle Command Line Interface.

Usage:
    rurtle [options] [<file>...]

Options:
    -h, --help                  Show this help.
    -o <file>, --output <file>  Set the output file.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_file: Vec<String>,
    flag_output: Option<String>,
}

#[cfg(feature = "cli")]
fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let mut environ = {
        let screen = if args.flag_output.is_none() {
            graphic::TurtleScreen::new((640, 640), "Rurtle")
        } else {
            // Screen to generate the test data for integration tests
            graphic::TurtleScreen::new_headless((800, 400), "Rurtle")
        };
        let screen = screen.unwrap_or_else(|err| {
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

    for filename in &args.arg_file {
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
    let rl = Arc::new(Mutex::new(Editor::<()>::new()));

    // Thread to do the blocking read so we can keep updating the window in the
    // main thread
    // Clone the readline so we can move the clone into the thread.
    let crl = rl.clone();
    let guard = thread::spawn(move || {
        loop {
            let input = crl.lock().unwrap().readline(PROMPT);
            match input {
                Ok(string) => tx.send(string).unwrap(),
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                },
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
            rl.lock().unwrap().add_history_entry(&source);
            match environ.eval_source(&source) {
                Ok(ref v) if *v != Value::Nothing => println!("{}", v),
                Err(e) => println!("{}: {}", e.description(), e),
                _ => (),
            }
        }
        let mut screen = environ.get_turtle().get_screen();
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

    if let Some(filename) = args.flag_output {
        environ.get_turtle().get_screen().draw_and_update();
        let screenshot = environ.get_turtle().get_screen().screenshot();
        let test_file = fs::File::open(&filename);
        if test_file.is_ok() {
            println!("{} already exists!", filename);
            process::exit(1);
        }
        let mut file = fs::File::create(&filename).unwrap();
        screenshot.save(&mut file, image::PNG).unwrap();
        println!("Saved to {}", filename);
        return
    }
}


#[cfg(not(feature = "cli"))]
fn main() {
    println!(r" __                                                                 ");
    println!(r"/  \                                                                ");
    println!(r"|  |                                                                ");
    println!(r"@  @  It looks like you want to use the Rurtle CLI...               ");
    println!(r"|  |     ...the Rurtle CLI requires the 'cli' feature enabled'...   ");
    println!(r"|| ||       ...would you like help with that?                       ");
    println!(r"|\_/|                                                               ");
    println!(r"\___/             <Yes>    <No>                                     ");
}
