// Not all tests use every single support function, so we get dead code
// warnings.
#![allow(dead_code)]
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

/// Compute the distance between two vectors, that is ||a-b||.
fn distance(a: [u8; 4], b: [u8; 4]) -> f32 {
    let norm_squared = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x as i32 - y as i32).pow(2))
        .fold(0, ::std::ops::Add::add);
    (norm_squared as f32).sqrt()
}

/// Return all neighbors in a 5x5 square surrounding the given coordinates.
fn neighbors(x: u32, y: u32) -> Vec<(u32, u32)> {
    let (ix, iy) = (x as i32, y as i32);
    (-2i32..3)
        .flat_map(|dx| (-2i32..3).map(|dy| (ix + dx, iy + dy)).collect::<Vec<(i32, i32)>>())
        .filter_map(|(x, y)| if x >= 0 && y >= 0 { Some((x as u32, y as u32)) } else { None })
        .collect()
}

/// Compare two images.
///
/// # Algorithm
///
/// For each pixel (x, y) in the source image, a 5x5 square in the second
/// image, centered around the same (x, y) coordinates is searched. If any of
/// those 25 pixels has a color vector that has a small distance to the source
/// vector, the pixel is matched. If all pixels are matched, the images are
/// equal.
pub fn image_eq(a: &image::DynamicImage, b: &image::DynamicImage) -> bool {
    use self::image::GenericImage;
    a.dimensions() == b.dimensions() && image_eq_impl(a, b) && image_eq_impl(b, a)
}

fn image_eq_impl(a: &image::DynamicImage, b: &image::DynamicImage) -> bool {
    use self::image::GenericImage;
    a
        .pixels()
        .all(|(x, y, pixel)|
             neighbors(x, y).into_iter().any(
                 |(nx, ny)| distance(pixel.data, b.get_pixel(nx, ny).data) < 0.2))
}
