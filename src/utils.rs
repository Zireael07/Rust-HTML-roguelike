use std::cmp::{max, min};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    NoDir,
	E,
	ENE,
	NE,
	NNE,
	N,
	NNW,
	NW,
	WNW,
	W,
	WSW,
	SW,
	SSW,
	S,
	SSE,
	SE,
	ESE,
}

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
    return max((sx-tx).abs(), (sy-ty).abs());
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

pub fn dir(start: &Point, end: &Point) -> Direction {
    let deltaX = (end.x-start.x).abs();
	let deltaY = (end.y-start.y).abs();
    
    match (start, end) {
        //https://doc.rust-lang.org/rust-by-example/flow_control/match/guard.html
        (start, end) if end.x > start.x && end.y == start.y => return Direction::E,
        (start, end) if end.x > start.x && end.y < start.y => {
            match (deltaX, deltaY) {
                (deltaX, deltaY) if deltaX > deltaY => return Direction::ENE,
                (deltaX, deltaY) if deltaX == deltaY => return Direction::NE,
                _ => return Direction::NNE,
            }
        },
        (start, end) if end.x == start.x && end.y < start.y => return Direction::N,
        (start, end) if end.x < start.x && end.y < start.y => {
            match (deltaX, deltaY) {
                (deltaX, deltaY) if deltaY > deltaX => return Direction::NNW,
                (deltaX, deltaY) if deltaX == deltaY => return Direction::NW,
                _ => return Direction::WNW,
            }
        },
        (start, end) if end.x < start.x && end.y == start.y => return Direction::W,
        (start, end) if end.x < start.x && end.y > start.y => {
            match (deltaX, deltaY) {
                (deltaX, deltaY) if deltaX > deltaY => return Direction::WSW,
                (deltaX, deltaY) if deltaX == deltaY => return Direction::SW,
                _ => return Direction::SSW,
            }
        },
        (start, end) if end.x == start.x && end.y > start.y => return Direction::S,
        (start, end) if end.x > start.x && end.y > start.y => {
            match (deltaX, deltaY) {
                (deltaX, deltaY) if deltaY > deltaX => return Direction::SSE,
                (deltaX, deltaY) if deltaX == deltaY => return Direction::SE,
                _ => return Direction::ESE,
            }
        }
        _ => return Direction::NoDir //dummy
    }
}