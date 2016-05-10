// Not all tests use every single support function, so we get dead code
// warnings.
#![allow(dead_code)]
use rurtle;
pub extern crate image;
extern crate nalgebra as na;

use std::ops::Add;
use self::na::Vector3;


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

fn rasterize_image(img: &mut image::DynamicImage) -> Vec<Vector3<f64>> {
    const TILE_WIDTH: u32 = 75;
    const TILE_HEIGHT: u32 = 75;
    use self::image::GenericImage;
    let (width, height) = img.dimensions();
    let (max_cols, max_rows) = (width / TILE_WIDTH, height / TILE_HEIGHT);
    let mut result = Vec::new();
    for row in 0..max_rows {
        for col in 0..max_cols {
            let tile = img.sub_image(col * TILE_WIDTH,
                                     row * TILE_HEIGHT,
                                     TILE_WIDTH,
                                     TILE_HEIGHT);
            let vec: Vector3<f64> = tile
                .pixels()
                .fold(na::zero(), |v, (_, _, data)|
                      v + Vector3::new(data[0] as f64, data[1] as f64, data[2] as f64));
            result.push(vec / tile.width() as f64 / tile.height() as f64);
        }
    }
    result
}

/// Compare two images.
///
/// # Algorithm
///
/// The images are rasterized into large tiles. Then the average RGB vector of
/// each tile is built and the norm of the difference of corresponding vectors
/// is accumulated. If this norm is below a threshold, the images are considered
/// equal.
pub fn image_eq(a: &mut image::DynamicImage, b: &mut image::DynamicImage) -> bool {
    use self::image::GenericImage;
    if a.dimensions() != b.dimensions() {
        return false;
    }
    let a_raster = rasterize_image(a);
    let b_raster = rasterize_image(b);
    let diff = a_raster
        .iter().zip(b_raster.iter())
        .map(|(&a_tile, &b_tile)| na::norm(&(a_tile - b_tile)))
        .fold(0.0, Add::add);
    diff < 1.0
}
