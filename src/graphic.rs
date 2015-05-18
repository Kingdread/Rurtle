//! This is the backend for turtle graphics. It's a wrapper around glium,
//! dealing with shaders etc. and providing a few high-level methods upon which
//! turtles can be built.
//!
//! # The Coordinate grid
//!
//! Unlike in other graphic libraries, the origin (0, 0) is in the middle of the
//! whole canvas. Positive x/y coordinates go right/up and negative ones
//! left/down. The canvas does have as many pixels as the window size, thus it
//! is possible to increase the section shown by resizing the window. Scrolling
//! and zooming are currently not supported.
//!
//! # Drawing and events
//!
//! To stay flexible, `TurtleScreen` has no built-in event loop. To redraw the
//! screen, use the `draw_and_update`-function. To handle events such as mouse
//! clicks, use `handle_events`.
//!
//! # Example
//!
//! ```
//! # use rurtle::graphic::{TurtleScreen, color};
//! let mut screen = TurtleScreen::new((640, 480), "Rurtle");
//! screen.add_line((0, 0), (50, 50), color::BLACK);
//! screen.turtle_position = (50, 50);
//! screen.turtle_orientation = 315.0;
//! screen.draw_and_update();
//! ```
extern crate glium;
use glium::Surface;

/// A Point to pass around to shaders.
#[derive(Copy, Clone)]
struct Point {
    coords: [f32; 2],
}
implement_vertex!(Point, coords);

/// Source for the vertex shader in the OpenGL shader language
const VERTEX_SHADER: &'static str = include_str!("shaders/vertex.glsl");
/// Source for the fragment shader in the OpenGL shader language
const FRAGMENT_SHADER: &'static str = include_str!("shaders/fragment.glsl");

/// Module for color aliases
pub mod color {
    /// Alias for a 4-f32 tuple, representing the colors as RGB values and the alpha
    /// channel
    pub type Color = (f32, f32, f32, f32);
    pub const BLACK: Color = (0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = (1.0, 1.0, 1.0, 1.0);
    pub const RED: Color = (1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = (0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = (0.0, 0.0, 1.0, 1.0);
}

/// A Line is defined via startpoint, endpoint and a color
struct Line(i32, i32, i32, i32, color::Color);


/// A `TurtleScreen` is a window that houses a turtle. It provides some graphic
/// methods, but you should use a `Turtle` instead.
pub struct TurtleScreen {
    window: glium::backend::glutin_backend::GlutinFacade,
    program: glium::Program,
    lines: Vec<Line>,
    _is_closed: bool,
    /// The position of the turtle on the canvas
    pub turtle_position: (i32, i32),
    /// The color of the turtle
    pub turtle_color: color::Color,
    /// The orientation of the turtle in degrees where 0° is north and positive
    /// degrees count counter-clockwise
    pub turtle_orientation: f32,
}

impl TurtleScreen {
    /// Create a new `TurtleScreen` with the given size and window title.
    ///
    /// # Panics
    ///
    /// Panics if something in the underlaying glium window creation fails.
    pub fn new(size: (u32, u32), title: &str) -> TurtleScreen {
        use glium::DisplayBuild;
        let builder = glium::glutin::WindowBuilder::new()
            .with_title(title.to_string())
            .with_dimensions(size.0, size.1)
            .build_glium();
        let window = match builder {
            Err(error) => panic!(format!("Window creation failed: {}", error)),
            Ok(win) => win,
        };
        let program_builder = glium::Program::from_source(
            &window, VERTEX_SHADER, FRAGMENT_SHADER, None);
        let program = match program_builder {
            Err(error) => panic!(format!("Program creation failed: {}", error)),
            Ok(prg) => prg,
        };
        TurtleScreen {
            window: window,
            program: program,
            lines: Vec::new(),
            _is_closed: false,
            turtle_position: (0, 0),
            turtle_color: color::BLACK,
            turtle_orientation: 0.0,
        }
    }

    /// Add a line to the collection, going from point start to point end
    pub fn add_line(&mut self, start: (i32, i32), end: (i32, i32), color: color::Color) {
        self.lines.push(Line(start.0, start.1, end.0, end.1, color));
    }

    /// Draw everything and update the screen
    pub fn draw_and_update(&self) {
        let mut frame = self.window.draw();
        frame.clear_color(1.0, 1.0, 1.0, 1.0);
        for line in self.lines.iter() {
            self.draw_line(&mut frame, line);
        }
        self.draw_turtle(&mut frame);
        frame.finish();
    }

    fn draw_line(&self, frame: &mut glium::Frame, line: &Line) {
        // It's probably pretty inefficient to call this function for every line
        // and it would be better to build the whole vertices list in one go,
        // but this way it's easier to color each line segment differently. I
        // will change it as soon as I learn more about the whole GLSL thing.
        use std::default::Default;

        let Line(x1, y1, x2, y2, color) = *line;
        let (width, height) = frame.get_dimensions();
        let vertex_buffer = glium::VertexBuffer::new(
            &self.window,
            vec![
                Point { coords: [x1 as f32, y1 as f32] },
                Point { coords: [x2 as f32, y2 as f32] },
            ]
        );
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);
        let matrix = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let uniforms = uniform! { icolor: color, matrix: matrix };
        frame.draw(&vertex_buffer, &indices, &self.program, &uniforms, &Default::default());
    }

    fn draw_turtle(&self, frame: &mut glium::Frame) {
        // The turtle consists of 4 points (let tx, ty = turtle_position):
        // A: tx, ty
        // B: tx + DELTA_OUT.0, ty - DELTA_OUT.1
        // C: tx + DELTA_MID.0, ty + DELTA_MID.1
        // D: tx + DELTA_OUT.0, ty + DELTA_OUT.1
        //     A
        //
        //     B
        //  C     D
        const DELTA_MID: (f32, f32) = (0.0, -10.0);
        const DELTA_OUT: (f32, f32) = (7.0, -13.0);

        let (tx, ty) = (self.turtle_position.0 as f32, self.turtle_position.1 as f32);
        let orientation_rad = ::std::f32::consts::PI * self.turtle_orientation / 180.0;
        let sin_d = orientation_rad.sin();
        let cos_d = orientation_rad.cos();

        // See http://en.wikipedia.org/wiki/Rotation_%28mathematics%29#Two_dimensions
        // for an explanation of the formula.
        // Again, it would probably be better to do it in the vertex shader...
        let vertex_buffer = glium::VertexBuffer::new(
            &self.window,
            vec![
                Point { coords: [tx, ty] },
                Point { coords: [
                    tx + (-DELTA_OUT.0 * cos_d - DELTA_OUT.1 * sin_d),
                    ty + (-DELTA_OUT.0 * sin_d + DELTA_OUT.1 * cos_d),
                    ] },
                Point { coords: [
                    tx + (DELTA_MID.0 * cos_d - DELTA_MID.1 * sin_d),
                    ty + (DELTA_MID.0 * sin_d + DELTA_MID.1 * cos_d),
                    ] },
                Point { coords: [
                    tx + (DELTA_OUT.0 * cos_d - DELTA_OUT.1 * sin_d),
                    ty + (DELTA_OUT.0 * sin_d + DELTA_OUT.1 * cos_d),
                    ] },
                ]
                );
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);
        let (width, height) = frame.get_dimensions();
        let matrix = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
            ];
        let uniforms = uniform! { icolor: self.turtle_color, matrix: matrix };
        frame.draw(&vertex_buffer, &indices, &self.program, &uniforms, &Default::default());
    }

    /// Poll the window's events and handle them
    pub fn handle_events(&mut self) {
        use glium::glutin::Event;
        for event in self.window.poll_events() {
            match event {
                Event::Closed => {
                    self._is_closed = true;
                    self.window.get_window().unwrap().hide();
                },
                _ => (),
            }
        }
    }

    /// Return if the window has been closed
    pub fn is_closed(&self) -> bool {
        return self._is_closed || self.window.is_closed();
    }
}
