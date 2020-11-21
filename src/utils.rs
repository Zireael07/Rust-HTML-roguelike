use std::cmp::{max, min};
use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    pub fn new(x:i32, y:i32) -> Point {
        return Point{x, y};
    }
}

//aka Chebyshev
pub fn distance2d_chessboard(sx: i32, sy: i32, tx: i32, ty: i32) -> i32 {
    return max((sy-sx).abs(), (ty-tx).abs());
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