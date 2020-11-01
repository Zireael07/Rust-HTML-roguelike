use std::cmp::{max, min};


#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    pub fn new(x:i32, y:i32) -> Point {
        return Point{x, y};
    }
}

// We're storing all the tiles in one big array, so we need a way to map an X,Y coordinate to
// a tile. Each row is stored sequentially (so 0..20, 21..40, etc.). This takes an x/y and returns
// the array index.
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 20) + x as usize
}

// It's a great idea to have a reverse mapping for these coordinates. This is as simple as
// index % 20 (mod 20), and index / 20
pub fn idx_xy(idx: usize) -> (i32, i32) {
    (idx as i32 % 20, idx as i32 / 20)
}

#[allow(dead_code)]
pub fn distance2d_squared(start: &Point, end: &Point) -> f32 {
    let dx = (max(start.x, end.x) - min (start.x, end.x)) as f32;
    let dy = (max(start.y, end.y) - min (start.y, end.y)) as f32;
    return (dx * dx) + (dy * dy);
}

#[allow(dead_code)]
pub fn distance2d(start: &Point, end: &Point) -> f32 {
    let dsq = distance2d_squared(start, end);
    return f32::sqrt(dsq);
}