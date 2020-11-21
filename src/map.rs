use serde::{Serialize, Deserialize};
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Floor = 0,
    Wall = 1,
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<u8>, //Vec<u8> can be passed by wasm_bindgen
    pub width: u32,
    pub height: u32,
    blocked: Vec<bool>,
    pub revealed_tiles : Vec<bool>,
}

impl Map {
    pub fn new(w:u32, h:u32) -> Map {
        let mut blocked = Vec::new();
        let mut revealed = Vec::new();
        for _i in 0 .. (w*h) {
            blocked.push(false);
            revealed.push(false);
        }
        let tiles = vec![Cell::Floor as u8; (w * h) as usize];

        return Map{width: w, height: h, tiles: tiles, blocked: blocked, revealed_tiles: revealed};
    }

    // We're storing all the tiles in one big array, so we need a way to map an X,Y coordinate to
    // a tile. Each row is stored sequentially (so 0..20, 21..40, etc.). This takes an x/y and returns
    // the array index.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    // It's a great idea to have a reverse mapping for these coordinates. This is as simple as
    // index % 20 (mod 20), and index / 20
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width as i32, idx as i32 / self.width as i32)
    }

    //blocked for pathfinding (eg. by other entities)
    pub fn set_tile_blocked(&mut self, idx : i32) {
        self.blocked[idx as usize] = true;
    }

    pub fn clear_tile_blocked(&mut self, idx : i32) {
        self.blocked[idx as usize] = false;
    }

    pub fn is_tile_blocked(&self, idx: i32) -> bool {
        return self.blocked[idx as usize];
    }

    pub fn is_tile_walkable(&self, x:i32, y:i32) -> bool {
        let idx = (y * self.width as i32) + x;
        return self.tiles[idx as usize] == Cell::Floor as u8;
    }

    pub fn is_tile_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width as i32-1 || y < 1 || y > self.height as i32-1 { return false; }
        let idx = (y * self.width as i32) + x;
        return !self.is_tile_blocked(idx);
    }

}