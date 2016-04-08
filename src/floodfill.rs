//! Floodfilling for Turtle graphics.
//!
//! # Algorithm
//!
//! First, the image is rasterized and a normal floodfill is performed on the
//! resulting image. Then the colored "blob" is saved and copied onto the
//! turtle window
use image::{self, GenericImage};
use bit_vec::BitVec;

/// Internal structure to simulate a 2d grid of true/false values
/// Faster than using a `HashSet` with (x, y) keys.
/// All elements are set to false initially.
struct BitMap2d {
    storage: BitVec,
    height: u32,
}

impl BitMap2d {
    fn new(width: u32, height: u32) -> BitMap2d {
        BitMap2d {
            storage: BitVec::from_elem(width as usize * height as usize, false),
            height: height,
        }
    }

    #[inline]
    fn calculate_index(&self, x: u32, y: u32) -> usize {
        x as usize * self.height as usize + y as usize
    }

    fn set(&mut self, x: u32, y: u32) {
        let index = self.calculate_index(x, y);
        self.storage.set(index, true);
    }

    /*
    fn clear(&mut self, x: u32, y: u32) {
        let index = self.calculate_index(x, y);
        self.storage.set(index, false);
    }
    */

    fn get(&mut self, x: u32, y: u32) -> bool {
        let index = self.calculate_index(x, y);
        self.storage.get(index).expect("Index out of bounds")
    }
}

/// Floodfill the given image, starting at the given `source` point and coloring
/// everything to `color`. Returns a Patch that contains the given colorized blob
/// with a transparent background. Also returns the (x, y) coordinates of the
/// upper left corner of the patch rectangle.
pub fn floodfill(img: &image::DynamicImage, start: (u32, u32), color: (u8, u8, u8, u8))
                 -> (u32, u32, image::DynamicImage)
{
    let (width, height) = img.dimensions();
    let mut result = Vec::new();
    let mut visited = BitMap2d::new(width, height);
    let mut queue = Vec::new();
    let source_color = img.get_pixel(start.0, start.1).data;
    let target_color = [color.0, color.1, color.2, color.3];
    queue.push(start);
    let mut neighbors = Vec::with_capacity(4);
    while let Some(point) = queue.pop() {
        let (x, y) = point;
        if source_color != img.get_pixel(x, y).data { continue }
        // Checking only 4 neighbors
        neighbors.clear();
        if x < width - 1 { neighbors.push((x+1, y)) };
        if x > 0 { neighbors.push((x-1, y)) };
        if y < height - 1 { neighbors.push((x, y+1)) };
        if y > 0 { neighbors.push((x, y-1)) };
        for (nx, ny) in neighbors.iter().cloned() {
            if visited.get(nx, ny) { continue };
            queue.push((nx, ny));
            visited.set(nx, ny);
        }
        result.push(point);
        visited.set(x, y);
    }
    let (min_x, max_x, min_y, max_y) = find_min_max(&result);
    let (patch_width, patch_height) = (max_x - min_x + 1, max_y - min_y + 1);
    let mut image = image::DynamicImage::new_rgba8(patch_width, patch_height);
    for (x, y) in result {
        image.put_pixel(x - min_x, y - min_y, image::Rgba { data: target_color } );
    }
    (min_x, min_y, image)
}

/// Takes a list of `(x, y)` coordinates and returns `(min_x, max_x, min_y, max_y)`
fn find_min_max(points: &[(u32, u32)]) -> (u32, u32, u32, u32) {
    let mut min_x = ::std::u32::MAX;
    let mut max_x = ::std::u32::MIN;
    let mut min_y = ::std::u32::MAX;
    let mut max_y = ::std::u32::MIN;
    for &(x, y) in points {
        if x < min_x { min_x = x };
        if x > max_x { max_x = x };
        if y < min_y { min_y = y };
        if y > max_y { max_y = y };
    }
    (min_x, max_x, min_y, max_y)
}
