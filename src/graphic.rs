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
//! screen.add_line((0.0, 0.0), (50.0, 50.0), color::BLACK);
//! screen.turtle_position = (50.0, 50.0);
//! screen.turtle_orientation = 315.0;
//! screen.draw_and_update();
//! ```
use image::{self, GenericImage};
use glium::{self, Surface};
use glium_text;
use na;
use std::io;
use super::floodfill as ff;

/// A Point to pass around to shaders.
#[derive(Copy, Clone)]
struct Point {
    coords: [f32; 2],
    color: [f32; 4],
}
implement_vertex!(Point, coords, color);

#[derive(Copy, Clone)]
struct FerrisPoint {
    coords: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(FerrisPoint, coords, tex_coords);

/// Source for the vertex shader in the OpenGL shader language
const VERTEX_SHADER: &'static str = include_str!("shaders/vertex.glsl");
/// Source for the fragment shader in the OpenGL shader language
const FRAGMENT_SHADER: &'static str = include_str!("shaders/fragment.glsl");
/// Ferris image bytes
const FERRIS_BYTES: &'static [u8] = include_bytes!("ferris.png");
/// Ferris vertex shader source
const FERRIS_VERTEX: &'static str = include_str!("shaders/ferris_vertex.glsl");
/// Ferris fragment shader
const FERRIS_FRAGMENT: &'static str = include_str!("shaders/ferris_fragment.glsl");
const PATCH_VERTEX: &'static str = include_str!("shaders/patch_vertex.glsl");
const PATCH_FRAGMENT: &'static str = include_str!("shaders/patch_fragment.glsl");
const FONT_DATA: &'static [u8] = include_bytes!("dejavusansmono.ttf");

type ScaleMatrix = [[f32; 4]; 4];

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

    /// Convert a Color (4-tuple of f32) to a color array ([f32; 4]). Useful for
    /// sending it to shaders.
    #[inline]
    pub fn to_array(color: Color) -> [f32; 4] {
        [color.0, color.1, color.2, color.3]
    }
}

/// A Line is defined via startpoint, endpoint and a color
struct Line(f32, f32, f32, f32, color::Color);
/// A Text is defined via anchor point, angle, color and text
struct Text(f32, f32, f32, color::Color, String);
/// A filled area is defined via a patch texture and a starting point
struct Fill(f32, f32, glium::texture::Texture2d);

/// Enum for every possible shape object
// We need this for a Vec<Shape> so that we can store the original order of
// every drawing. It's still easier to have seperate structs for the relevant
// data so we can have a function
//     draw_text(&mut Frame, &Text)
// instead of
//     draw_text(&mut Frame, f32, f32, f32, color::Color, &String)
// or
//     draw_text(&mut Frame, &Shape), which would require pattern matching twice
enum Shape {
    Line(Line),
    Text(Text),
    Fill(Fill),
}

/// A `TurtleScreen` is a window that houses a turtle. It provides some graphic
/// methods, but you should use a `Turtle` instead.
pub struct TurtleScreen {
    window: glium::backend::glutin_backend::GlutinFacade,
    program: glium::Program,
    shapes: Vec<Shape>,
    _is_closed: bool,
    ferris: glium::texture::Texture2d,
    ferris_program: glium::Program,
    patch_program: glium::Program,
    text_system: glium_text::TextSystem,
    font: glium_text::FontTexture,
    /// The position of the turtle on the canvas
    pub turtle_position: (f32, f32),
    /// The color of the turtle
    pub turtle_color: color::Color,
    /// The orientation of the turtle in degrees where 0° is north and positive
    /// degrees count counter-clockwise
    pub turtle_orientation: f32,
    /// If this is set to true, the turtle itself won't be drawn
    pub turtle_hidden: bool,
    /// Background color of the turtle screen
    pub background_color: color::Color,
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
            Err(error) => panic!("Window creation failed: {}", error),
            Ok(win) => win,
        };
        let program_builder = glium::Program::from_source(
            &window, VERTEX_SHADER, FRAGMENT_SHADER, None);
        let program = match program_builder {
            Err(error) => panic!("Program creation failed: {}", error),
            Ok(prg) => prg,
        };
        let ferris_image = image::load(io::Cursor::new(FERRIS_BYTES),
                                       image::ImageFormat::PNG).unwrap();
        let ferris_texture = glium::texture::Texture2d::new(&window, ferris_image).unwrap();
        let ferris_program = glium::Program::from_source(&window, FERRIS_VERTEX,
                                                         FERRIS_FRAGMENT, None) .unwrap();
        let patch_program = glium::Program::from_source(&window, PATCH_VERTEX,
                                                        PATCH_FRAGMENT, None).unwrap();
        let text_system = glium_text::TextSystem::new(&window);
        let font = glium_text::FontTexture::new(&window,
                                                io::Cursor::new(FONT_DATA), 24).unwrap();
        TurtleScreen {
            window: window,
            program: program,
            shapes: Vec::new(),
            _is_closed: false,
            ferris: ferris_texture,
            ferris_program: ferris_program,
            patch_program: patch_program,
            text_system: text_system,
            font: font,
            turtle_position: (0.0, 0.0),
            turtle_color: color::BLACK,
            turtle_orientation: 0.0,
            turtle_hidden: false,
            background_color: color::WHITE,
        }
    }

    /// Add a line to the collection, going from point start to point end
    pub fn add_line(&mut self, start: (f32, f32), end: (f32, f32), color: color::Color) {
        self.shapes.push(Shape::Line(Line(start.0, start.1, end.0, end.1, color)));
    }

    /// Add a new text to the screen
    pub fn add_text(&mut self, anchor: (f32, f32), angle: f32, color: color::Color, text: &str) {
        self.shapes.push(Shape::Text(Text(anchor.0, anchor.1, angle, color, text.to_string())));
    }

    /// Floodfill the image at the given point with the given color
    pub fn floodfill(&mut self, point: (f32, f32), color: color::Color) {
        // we floodfill with the turtle not shown
        let original_state = self.turtle_hidden;
        self.turtle_hidden = true;
        self.draw_and_update();
        let image = self.screenshot();
        self.turtle_hidden = original_state;
        self.draw_and_update();
        // point is given in turtle coordinates with (0,0) being in the middle, we
        // need to translate it to picture coordinates
        let (width, height) = image.dimensions();
        let (adj_x, adj_y) = ((width as f32 / 2. + point.0) as u32,
                              // minus here because the image coordinates have the
                              // y-axis downwards while turtle coordinates have the
                              // y-axis upwars
                              (height as f32 / 2. - point.1) as u32);
        let translated_color = {
            let (r, g, b, a) = color;
            const MAX: f32 = ::std::u8::MAX as f32;
            ((MAX * r) as u8, (MAX * g) as u8, (MAX * b) as u8, (MAX * a) as u8)
        };
        let (px, py, patch) = ff::floodfill(&image, (adj_x, adj_y), translated_color);
        // We need to translate back the start coordinates
        let (trans_x, trans_y) = (px as f32 - width as f32 / 2.,
                                  height as f32 / 2. - py as f32);
        self.shapes.push(Shape::Fill(
            Fill(trans_x, trans_y,
                 glium::texture::Texture2d::new(&self.window, patch).unwrap())));
    }

    /// Remove all drawn lines. Note that this does not change the turtle's
    /// position, color or orientation.
    pub fn clear(&mut self) {
        self.shapes.clear();
    }

    /// Draw everything and update the screen
    pub fn draw_and_update(&self) {
        let mut frame = self.window.draw();
        {
            let (br, bg, bb, ba) = self.background_color;
            frame.clear_color(br, bg, bb, ba);
        }
        let (width, height) = frame.get_dimensions();
        let matrix = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / height as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        for shape in &self.shapes {
            match *shape {
                Shape::Line(ref l) => self.draw_line(&mut frame, l, matrix),
                Shape::Text(ref t) => self.draw_text(&mut frame, t),
                Shape::Fill(ref f) => self.draw_fill(&mut frame, f, matrix),
            }
        }
        if !self.turtle_hidden {
            self.draw_turtle(&mut frame, matrix);
        }
        frame.finish().unwrap();
    }

    fn draw_fill(&self, frame: &mut glium::Frame, fill: &Fill, matrix: ScaleMatrix) {
        let Fill(x, y, ref texture) = *fill;
        let (width, height) = (texture.get_width() as f32,
                               texture.get_height().unwrap() as f32);
        let vertex_buffer = glium::VertexBuffer::new(
            &self.window,
            &vec![
                // Bottom left corner
                FerrisPoint { coords: [x, y - height], tex_coords: [0., 0.] },
                // Bottom right corner
                FerrisPoint { coords: [x + width, y - height], tex_coords: [1., 0.] },
                // Top right corner
                FerrisPoint { coords: [x + width, y], tex_coords: [1., 1.] },
                // Top left corner
                FerrisPoint { coords: [x, y], tex_coords: [0., 1.] },
        ]);
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);
        let uniforms = uniform! {
            matrix: matrix,
            texture_data: texture,
        };
        frame.draw(&vertex_buffer.unwrap(), &indices, &self.patch_program, &uniforms,
                   &Default::default()).unwrap();
    }

    fn draw_line(&self, frame: &mut glium::Frame, line: &Line, matrix: ScaleMatrix) {
        use std::default::Default;
        use self::color::to_array;
        let mut points: Vec<Point> = Vec::new();
        let Line(x1, y1, x2, y2, color) = *line;
        points.push(Point { coords: [x1, y1], color: to_array(color) });
        points.push(Point { coords: [x2, y2], color: to_array(color) });
        let vertex_buffer = glium::VertexBuffer::new(&self.window, &points);
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);
        let uniforms = uniform! { matrix: matrix };
        frame.draw(&vertex_buffer.unwrap(), &indices, &self.program, &uniforms, &Default::default())
            .unwrap();
    }

    fn draw_text(&self, frame: &mut glium::Frame, text: &Text) {
        const FONT_SIZE: f32 = 12.;
        let Text(pos_x, pos_y, angle_deg, text_color, ref data) = *text;
        // Convert to radians
        let angle = ::std::f32::consts::PI * angle_deg / 180.;
        let sin_d = angle.sin();
        let cos_d = angle.cos();
        let text = glium_text::TextDisplay::new(&self.text_system, &self.font, data);
        let (width, height) = frame.get_dimensions();
        // Note that this is not column-major layout
        let rotation_matrix = na::Mat4::new(
            cos_d, -sin_d, 0., 0.,
            sin_d, cos_d, 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.);
        let scale_matrix = na::Mat4::new(
            2. * FONT_SIZE / width as f32, 0., 0., 0.,
            0., 2. * FONT_SIZE / height as f32, 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.);
        let translate_matrix = na::Mat4::new(
            1., 0., 0., pos_x * 2. / width as f32,
            0., 1., 0., pos_y * 2. / height as f32,
            0., 0., 1., 0.,
            0., 0., 0., 1.);
        glium_text::draw(&text, &self.text_system, frame,
                         *(translate_matrix * scale_matrix * rotation_matrix).as_array(),
                         text_color);
    }

    fn draw_turtle(&self, frame: &mut glium::Frame, matrix: ScaleMatrix) {
        // WIDTH and HEIGHT specifiy the size in which Ferris should be drawn.
        // The aspect ratio should be kept, the original Ferris image has a
        // ratio of w:h 3:2
        const WIDTH: f32 = 36.;
        const HEIGHT: f32 = 24.;
        const DX: f32 = WIDTH / 2.;
        const DY: f32 = HEIGHT / 2.;

        let (tx, ty) = self.turtle_position;
        let orientation_rad = ::std::f32::consts::PI * self.turtle_orientation / 180.0;
        let sin_d = orientation_rad.sin();
        let cos_d = orientation_rad.cos();

        let rotation_matrix = [
            [cos_d, sin_d, 0., 0.],
            [-sin_d, cos_d, 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ];

        let vertex_buffer = glium::VertexBuffer::new(
            &self.window,
            &vec![
                // Bottom left corner
                FerrisPoint { coords: [tx - DX, ty - DY], tex_coords: [0., 0.] },
                // Bottom right corner
                FerrisPoint { coords: [tx + DX, ty - DY], tex_coords: [1., 0.] },
                // Top right corner
                FerrisPoint { coords: [tx + DX, ty + DY], tex_coords: [1., 1.] },
                // Top left corner
                FerrisPoint { coords: [tx - DX, ty + DY], tex_coords: [0., 1.] },
        ]);
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);
        let uniforms = uniform! {
            matrix: matrix,
            rotation_matrix: rotation_matrix,
            ferris_tex: &self.ferris,
            tip_x: tx,
            tip_y: ty,
        };
        frame.draw(&vertex_buffer.unwrap(), &indices, &self.ferris_program, &uniforms,
                   &Default::default()).unwrap();
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

    /// Return if the window has been closed. A closed window can only be
    /// detected if the window's events have been handled. Thus it is advised to
    /// use `handle_events()` before checking `is_closed()`.
    pub fn is_closed(&self) -> bool {
        self._is_closed
    }

    /// Return the current screen as an image
    pub fn screenshot(&self) -> image::DynamicImage {
        self.window.read_front_buffer()
    }
}
