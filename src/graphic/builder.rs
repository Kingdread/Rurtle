//! Abstractions over different ways to get a glium builders. Basically, this is
//! `glium::glutin::DisplayBuild` on steroids.
//!
//! We can't use `DisplayBuild` directly because it lacks methods for setting
//! certain attributes. This makes sense because not every builder supports all
//! the attributes, but we don't care about that, we just ignore the unsupported
//! ones. Thus it is not guaranteed that every builder respects the `with_title`
//! call (e.g. the headless render does not respect it).
use std::ops::Deref;
use std::rc::Rc;
use std::cell::RefCell;
use std::mem;
use glium::glutin::{WindowBuilder, HeadlessRendererBuilder, GlRequest};
use glium::texture::{RawImage2d, Texture2d};
use glium::framebuffer::{SimpleFrameBuffer, MultiOutputFrameBuffer};
use glium::uniforms::{MagnifySamplerFilter, Uniforms};
use glium::vertex::MultiVerticesSource;
use glium::index::IndicesSource;
use glium::{self, DisplayBuild, Surface, Rect, BlitTarget, DrawError, Program, DrawParameters};

/// Type of the facade returned by the builders.
pub type Facade = glium::backend::glutin_backend::GlutinFacade;
/// Type of the error returned by the builders.
pub type Err = glium::GliumCreationError<glium::glutin::CreationError>;

/// Type alias to facilitate the creation of windowed renderers.
pub type Window = WindowBuilder<'static>;
/// Type alias to facilitate the creation of headless renderers.
pub type Headless = HeadlessRendererBuilder<'static>;

/// An object that may create glium facades and provides more control than
/// `DisplayBuild`.
pub trait GliumFactory: Sized + DisplayBuild {
    /// Type of the renderer returned by this factory.
    type Rend: 'static + Renderer;

    /// Create a new builder with the given dimensions.
    fn new(width: u32, height: u32) -> Self;

    /// Request a specific title for the window.
    fn with_title(self, title: String)  -> Self;

    /// Sets how the backend should choose the OpenGL API and version.
    fn with_gl(self, request: GlRequest) -> Self;

    fn build_renderer(self) -> Result<Self::Rend, Err>;
}

impl<'a> GliumFactory for WindowBuilder<'static> {
    type Rend = WindowRenderer;

    fn new(width: u32, height: u32) -> WindowBuilder<'static> {
        WindowBuilder::new().with_dimensions(width, height)
    }

    fn with_title(self, title: String) -> WindowBuilder<'static> {
        self.with_title(title)
    }

    fn with_gl(self, request: GlRequest) -> Self {
        self.with_gl(request)
    }

    fn build_renderer(self) -> Result<WindowRenderer, Err> {
        let window = try!(self.build_glium());
        Ok(WindowRenderer {
            window: window,
            frame: None,
        })
    }
}

impl<'a> GliumFactory for HeadlessRendererBuilder<'a> {
    type Rend = HeadlessRenderer;

    fn new(width: u32, height: u32) -> HeadlessRendererBuilder<'a> {
        HeadlessRendererBuilder::new(width, height)
    }

    fn with_title(self, _: String)  -> Self {
        self
    }

    fn with_gl(self, request: GlRequest) -> Self {
        self.with_gl(request)
    }

    fn build_renderer(self) -> Result<HeadlessRenderer, Err> {
        let dimensions = self.dimensions;
        let backend = try!(self.build_glium());
        Ok(HeadlessRenderer {
            backend: backend,
            dimensions: dimensions,
            buffer: None,
        })
    }
}

/// Some backend that can be used to render.
///
/// This abstraction is needed because we have different targets for normal
/// renderers and headless renderes: For headless renderers, the framebuffer
/// size is fixed, so we draw onto an external texture. For windows however, the
/// framebuffer size depends on the window size (which is good), so there we
/// draw onto the window buffer. This abstraction makes it easier to not care
/// about that distinction.
pub trait Renderer: Deref<Target=Facade> {
    /// Get a reference to the buffer that should be used.
    fn draw(&mut self) -> MySurface;

    /// Finish the frame, possibly swap the buffers.
    fn finish(&mut self);

    /// Read the image data, i.e. take a screenshot.
    fn read(&self) -> RawImage2d<u8>;
}

impl glium::backend::Facade for Box<Renderer<Target=Facade>> {
    fn get_context(&self) -> &Rc<glium::backend::Context> {
        (**self).get_context()
    }
}

pub struct WindowRenderer {
    window: Facade,
    frame: Option<Rc<RefCell<glium::Frame>>>,
}

impl Deref for WindowRenderer {
    type Target = Facade;

    fn deref(&self) -> &Facade {
        &self.window
    }
}

impl Renderer for WindowRenderer {
    fn draw(&mut self) -> MySurface {
        let frame = Rc::new(RefCell::new(self.window.draw()));
        self.frame = Some(frame.clone());
        MySurface::Frame(frame)
    }

    fn finish(&mut self) {
        let mut frame = None;
        mem::swap(&mut frame, &mut self.frame);
        let frame = Rc::try_unwrap(frame.expect("No frame to finish, call draw() first!"));
        frame
            .ok()
            .expect("Lingering references to the internal frame!")
            .into_inner()
            .finish()
            .expect("Error swapping the framebuffers");
    }

    fn read(&self) -> RawImage2d<u8> {
        self.window.read_front_buffer()
    }
}

pub struct HeadlessRenderer {
    backend: Facade,
    dimensions: (u32, u32),
    buffer: Option<Rc<Texture2d>>,
}

impl Deref for HeadlessRenderer {
    type Target = Facade;

    fn deref(&self) -> &Facade {
        &self.backend
    }
}

impl Renderer for HeadlessRenderer {
    fn draw(&mut self) -> MySurface {
        let (width, height) = self.dimensions;
        let buffer = Rc::new(Texture2d::empty(&self.backend, width, height).unwrap());
        self.buffer = Some(buffer.clone());
        MySurface::Buffer(buffer)
    }

    fn finish(&mut self) {}

    fn read(&self) -> RawImage2d<u8> {
        self.buffer.as_ref().expect("No data yet available").read()
    }
}

macro_rules! implement {
    ($name:ident; $result:ty; $($p:ident: $t:ty),*) => {
        fn $name(&self, $($p: $t),*) -> $result {
            match *self {
                MySurface::Frame(ref ptr) => ptr.borrow().$name($($p),*),
                MySurface::Buffer(ref tex) => tex.as_surface().$name($($p),*),
            }
        }
    };
    (mut $name:ident; $result:ty; $($p:ident: $t:ty),*) => {
        fn $name(&mut self, $($p: $t),*) -> $result {
            match *self {
                MySurface::Frame(ref mut ptr) => ptr.borrow_mut().$name($($p),*),
                MySurface::Buffer(ref tex) => tex.as_surface().$name($($p),*),
            }
        }
    }
}

/// Enum to abstract away different surface implementations.
///
/// Since `glium::Surface` is not object-safe, we can't create `&mut Surface`,
/// so we need another way to have a single return type for both `glium::Frame`
/// and `SimpleFrameBuffer`. This enum reimplements the most important methods.
pub enum MySurface {
    Frame(Rc<RefCell<glium::Frame>>),
    Buffer(Rc<Texture2d>),
}

impl Surface for MySurface {
    implement!(mut clear; (); rect: Option<&Rect>, color: Option<(f32, f32, f32, f32)>, color_srgb: bool, depth: Option<f32>, stencil: Option<i32>);
    implement!(get_dimensions; (u32, u32););
    implement!(get_depth_buffer_bits; Option<u16>;);
    implement!(get_stencil_buffer_bits; Option<u16>;);
    fn draw<'a, 'b, V, I, U>(&mut self,
                             vb: V,
                             ib: I,
                             program: &Program,
                             uniforms: &U,
                             draw_parameters: &DrawParameters)
                             -> Result<(), DrawError>
        where V: MultiVerticesSource<'b>, I: Into<IndicesSource<'a>>, U: Uniforms
    {
        match *self {
            MySurface::Frame(ref mut ptr) => ptr.borrow_mut().draw(vb, ib, program, uniforms, draw_parameters),
            MySurface::Buffer(ref tex) => tex.as_surface().draw(vb, ib, program, uniforms, draw_parameters),
        }
    }
    implement!(blit_from_frame; (); source_rect: &Rect, target_rect: &BlitTarget, filter: MagnifySamplerFilter);
    implement!(blit_from_simple_framebuffer; (); source: &SimpleFrameBuffer, source_rect: &Rect, target_rect: &BlitTarget, filter: MagnifySamplerFilter);
    implement!(blit_from_multioutput_framebuffer; (); source: &MultiOutputFrameBuffer, source_rect: &Rect, target_rect: &BlitTarget, filter: MagnifySamplerFilter);
    fn blit_color<S>(&self,
                     source_rect: &Rect,
                     target: &S,
                     target_rect: &BlitTarget,
                     filter: MagnifySamplerFilter)
        where S: Surface
    {
        match *self {
            MySurface::Frame(ref ptr) => ptr.borrow().blit_color(source_rect, target, target_rect, filter),
            MySurface::Buffer(ref tex) => tex.as_surface().blit_color(source_rect, target, target_rect, filter),
        }
    }
}
