use rurtle;

/// Dimensions used for the test image
pub const DIMENSIONS: (u32, u32) = (800, 400);

pub fn screen() -> rurtle::TurtleScreen {
    rurtle::TurtleScreen::new_headless(DIMENSIONS, "Rurtle Test").unwrap()
}

pub fn turtle() -> rurtle::Turtle {
    rurtle::Turtle::new(screen())
}
