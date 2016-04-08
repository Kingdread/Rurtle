//! This module contains the actual implementation of the turtle concept.
//!
//! The `Turtle` starts in the middle of the screen and can be given commands
//! such as `forward(100)` or `left(90)`. While moving across the screen, the
//! turtle draws its path on the canvas. Based on this primitive movements, you
//! can build more complex commands and draw nice patterns.
//!
//! A `Turtle` always has a `TurtleScreen` owned. This `TurtleScreen` must be
//! given to `Turtle::new()`
//!
//! # Example
//!
//! ```no_run
//! use rurtle::graphic::TurtleScreen;
//! use rurtle::turtle::Turtle;
//! let screen = TurtleScreen::new((640, 480), "Turtle Demo").unwrap();
//! let mut turtle = Turtle::new(screen);
//! for _ in 0..4 {
//!     turtle.forward(100.0);
//!     turtle.right(90.0);
//! }
//! ```
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::mem;
use super::graphic::{TurtleScreen, color};

#[derive(Debug)]
enum PenState {
    PenUp,
    PenDown,
}

/// The `Turtle` struct is the thing that actually provides the methods to walk
/// on the screen
pub struct Turtle {
    screen: Rc<RefCell<TurtleScreen>>,
    data: Rc<RefCell<TurtleData>>,
    pen: PenState,
}

/// Internal data of the turtle
pub struct TurtleData {
    /// The position of the turtle on the canvas
    pub position: (f32, f32),
    /// The color of the turtle
    pub color: color::Color,
    /// The orientation of the turtle in degrees where 0° is north and positive
    /// degrees count counter-clockwise
    pub orientation: f32,
    /// If this is set to true, the turtle itself won't be drawn
    pub hidden: bool,
    /// Numeric id of the turtle
    pub id: usize,
}

impl Turtle {
    /// Construct a new Turtle. Moves the TurtleScreen.
    pub fn new(mut screen: TurtleScreen) -> Turtle {
        let data = Rc::new(RefCell::new(TurtleData {
            position: (0.0, 0.0),
            color: color::BLACK,
            orientation: 0.0,
            hidden: false,
            id: screen.counter(),
        }));
        screen.add_turtle(Rc::downgrade(&data));
        Turtle {
            screen: Rc::new(RefCell::new(screen)),
            data: data,
            pen: PenState::PenDown,
        }
    }

    /// Return a `Turtle` that shares the `TurtleScreen`.
    ///
    /// This returns a separate turtle, which has its own position, color,
    /// handle, ... but that uses the same `TurtleScreen`. This is currently
    /// the only way to share screens and clone turtles.
    pub fn procreate(&self) -> Turtle {
        let data = Rc::new(RefCell::new(TurtleData { 
            position: (0.0, 0.0),
            color: color::BLACK,
            orientation: 0.0,
            hidden: false,
            id: self.screen.borrow().counter(),
        }));
        self.screen.borrow_mut().add_turtle(Rc::downgrade(&data));
        Turtle {
            screen: self.screen.clone(),
            data: data,
            pen: PenState::PenDown,
        }
    }

    /// Move the turtle to the given position. Depending on whether the pen is
    /// up or down, also draw the line. This function is used internally to
    /// implement everything else
    fn goto(&mut self, x: f32, y: f32) {
        let start_position = self.data.borrow().position;
        if let PenState::PenDown = self.pen {
            self.screen.borrow_mut().add_line(start_position, (x, y), self.data.borrow().color);
        }
        self.data.borrow_mut().position = (x, y);
        self.screen.borrow_mut().draw_and_update();
    }

    /// Return a reference to the underlaying `TurtleScreen` object
    pub fn get_screen<'a>(&'a mut self) -> RefMut<'a, TurtleScreen> {
        self.screen.borrow_mut()
    }

    /// Turn the turtle by the given amount. Positive means counter-clockwise,
    /// negative means clockwise. The angle is given in degrees. This function
    /// is used internally.
    fn turn(&mut self, deg: f32) {
        let orientation = self.data.borrow().orientation;
        self.set_orientation(orientation + deg);
    }

    /// Take the length of a path and return the (delta_x, delta_y) attributes
    /// that you need to "walk" when heading in the current direction.
    fn length_to_vector(&self, length: f32) -> (f32, f32) {
        let orientation_rad = ::std::f32::consts::PI * self.data.borrow().orientation / 180.0;
        let delta_x = orientation_rad.sin() * length;
        let delta_y = orientation_rad.cos() * length;
        (-delta_x, delta_y)
    }

    /// Clear the screen. Note that this only removes the drawn lines, it does
    /// not change the turtle's position or orientation.
    pub fn clear(&mut self) {
        self.screen.borrow_mut().clear();
    }

    /// Move the turtle forward by the given length
    pub fn forward(&mut self, length: f32) {
        let (x, y) = self.data.borrow().position;
        let (dx, dy) = self.length_to_vector(length);
        self.goto(x + dx, y + dy);
    }

    /// Move the turtle backward by the given length
    pub fn backward(&mut self, length: f32) {
        let (x, y) = self.data.borrow().position;
        let (dx, dy) = self.length_to_vector(length);
        self.goto(x - dx, y - dy);
    }

    /// Turn the turtle left
    pub fn left(&mut self, deg: f32) {
        self.turn(deg);
    }

    /// Turn the turtle right
    pub fn right(&mut self, deg: f32) {
        self.turn(-deg);
    }

    /// "Lifts" the pen so that no lines are drawn anymore
    pub fn pen_up(&mut self) {
        self.pen = PenState::PenUp;
    }

    /// Sinks the pen again so that lines are drawn
    pub fn pen_down(&mut self) {
        self.pen = PenState::PenDown;
    }

    /// Set the turtle's color. New lines will be drawn using that color but
    /// existing lines will remain in their color. `red`, `green` and `blue` are
    /// given as floats in the range [0; 1], where 0 means nothing and 1 full
    /// (like #FF in HTML).
    pub fn set_color(&mut self, red: f32, green: f32, blue: f32) {
        self.data.borrow_mut().color = (red, green, blue, 1.0);
        self.screen.borrow_mut().draw_and_update();
    }

    /// Set the background color of the screen.
    pub fn set_background_color(&mut self, red: f32, green: f32, blue: f32) {
        self.screen.borrow_mut().background_color = (red, green, blue, 1.);
        self.screen.borrow_mut().draw_and_update();
    }

    /// Directly move the turtle to the given point without changing the
    /// direction. Draws a line if the pen is down. Note that the origin (0, 0)
    /// is in the center of the screen with positive coordinates being right/top
    /// and negative ones left/down.
    pub fn teleport(&mut self, x: f32, y: f32) {
        self.goto(x, y)
    }

    /// Set the turtle's orientation in degrees with 0 being faced north and
    /// positive degrees counting counter-clockwise.
    pub fn set_orientation(&mut self, deg: f32) {
        self.data.borrow_mut().orientation = deg % 360.0;
        self.screen.borrow_mut().draw_and_update();
    }

    /// Move the turtle to the origin and set its orientation to 0
    pub fn home(&mut self) {
        self.teleport(0.0, 0.0);
        self.set_orientation(0.0);
    }

    /// Return the turtle's orientation
    pub fn get_orientation(&self) -> f32 { self.data.borrow().orientation }
    /// Return the turtle's position
    pub fn get_position(&self) -> (f32, f32) { self.data.borrow().position }

    /// Hide the turtle so it won't be drawn on the screen
    pub fn hide(&mut self) {
        self.data.borrow_mut().hidden = true;
        self.screen.borrow_mut().draw_and_update();
    }

    /// Show the turtle again after it has been hidden
    pub fn show(&mut self) {
        self.data.borrow_mut().hidden = false;
        self.screen.borrow_mut().draw_and_update();
    }

    /// Returns true if the turtle is currently hidden
    pub fn is_hidden(&self) -> bool {
        self.data.borrow().hidden
    }

    /// Write the text on the screen. The lower-left corner of the Text starts
    /// where the turtle is.
    pub fn write(&mut self, text: &str) {
        let data = self.data.borrow();
        self.screen.borrow_mut().add_text(data.position, data.orientation, data.color, text);
    }

    /// Perform a floodfill at the current turtle position
    pub fn flood(&mut self) {
        // own scope to destroy the borrow as soon as possible
        let (position, color) = {
            let data = self.data.borrow();
            (data.position, data.color)
        };
        self.screen.borrow_mut().floodfill(position, color);
    }
}

impl Drop for Turtle {
    fn drop(&mut self) {
        // Zeroed memory is fine because TurtleData has no destructor, it just
        // contains POD (it could even be a C struct).
        let mut dummy = Rc::new(RefCell::new(unsafe { mem::zeroed() }));
        mem::swap(&mut self.data, &mut dummy);
        // dummy is now our real data. When we drop it, the weak references that
        // the TurtleScreen stores become invalid and will be cleaned up.
        mem::drop(dummy);
        self.screen.borrow_mut().cleanup();
    }
}
