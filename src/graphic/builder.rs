//! Abstractions over different ways to get a glium builders. Basically, this is
//! `glium::glutin::DisplayBuild` on steroids.
//!
//! We can't use `DisplayBuild` directly because it lacks methods for setting
//! certain attributes. This makes sense because not every builder supports all
//! the attributes, but we don't care about that, we just ignore the unsupported
//! ones. Thus it is not guaranteed that every builder respects the `with_title`
//! call (e.g. the headless render does not respect it).
use glium::glutin::{WindowBuilder, HeadlessRendererBuilder, GlRequest};
use glium::{self, DisplayBuild};

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
    /// Create a new builder with the given dimensions.
    fn new(width: u32, height: u32) -> Self;

    /// Request a specific title for the window.
    fn with_title(self, title: String)  -> Self;

    /// Sets how the backend should choose the OpenGL API and version.
    fn with_gl(self, request: GlRequest) -> Self;

    fn build_glium(self) -> Result<Facade, Err>;
}

impl<'a> GliumFactory for WindowBuilder<'static> {
    fn new(width: u32, height: u32) -> WindowBuilder<'static> {
        WindowBuilder::new().with_dimensions(width, height)
    }

    fn with_title(self, title: String) -> WindowBuilder<'static> {
        self.with_title(title)
    }

    fn with_gl(self, request: GlRequest) -> Self {
        self.with_gl(request)
    }

    fn build_glium(self) -> Result<Facade, Err> {
        DisplayBuild::build_glium(self)
    }
}

impl<'a> GliumFactory for HeadlessRendererBuilder<'a> {
    fn new(width: u32, height: u32) -> HeadlessRendererBuilder<'a> {
        HeadlessRendererBuilder::new(width, height)
    }

    fn with_title(self, _: String)  -> Self {
        self
    }

    fn with_gl(self, request: GlRequest) -> Self {
        self.with_gl(request)
    }

    fn build_glium(self) -> Result<Facade, Err> {
        DisplayBuild::build_glium(self)
    }
}
