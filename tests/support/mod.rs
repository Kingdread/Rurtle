use rurtle;
pub extern crate image;

/// Dimensions used for the test image
pub const DIMENSIONS: (u32, u32) = (800, 400);

pub fn screen() -> rurtle::TurtleScreen {
    rurtle::TurtleScreen::new_headless(DIMENSIONS, "Rurtle Test").unwrap()
}

pub fn turtle() -> rurtle::Turtle {
    rurtle::Turtle::new(screen())
}

pub fn environ() -> rurtle::Environment {
    rurtle::Environment::new(turtle())
}

pub fn image_eq(a: &image::DynamicImage, b: &image::DynamicImage) -> bool {
    use self::image::GenericImage;
    a.pixels().eq(b.pixels())
}
