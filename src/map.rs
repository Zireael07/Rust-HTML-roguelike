use serde::{Serialize, Deserialize};
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use super::fastnoise::*;

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

    //mapgen
    pub fn build_map() -> Map {
        let mut map = Map::new(20,20);

        //noise
        //generate noise
        let mut rng = rand::thread_rng();
        let mut noise = FastNoise::seeded(10001 as u64);
        //let mut noise = FastNoise::seeded(rng.gen_range(1, 65537) as u64);
        noise.set_noise_type(NoiseType::SimplexFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(1);
        noise.set_fractal_gain(0.5);
        noise.set_fractal_lacunarity(2.0);
        //noise.set_frequency(0.085);
        noise.set_frequency(0.45);

        for x in 0..20 {
            for y in 0..20 {
                let mut n = noise.get_noise(x as f32, y as f32);
                n = n*-255 as f32; //because defaults are vanishingly small
                //log!("{}", &format!("Noise: x{}y{} {}", x, y, n));
                if n > 125.0 || n < -125.0 {
                    let idx = map.xy_idx(x,y);
                    map.tiles[idx] = Cell::Wall as u8;
                    // opaque
                    //state.fov_data.set_transparent(x as usize, y as usize, false);
                } else {
                    //state.tiles[xy_idx(x,y)] = Cell:Floor as u8
                }
                //log!("{}", &format!("Tile: x{} y{} {}", x,y, state.tiles[xy_idx(x,y)]));
            }
        }
      


        // Make the boundaries walls
        for x in 0..20 {
            let mut idx = map.xy_idx(x, 0);
            map.tiles[idx] = Cell::Wall as u8;
            idx = map.xy_idx(x, 19);
            map.tiles[idx] = Cell::Wall as u8;
            //mark 'em as opaque
            //state.fov_data.set_transparent(x as usize, 0 as usize, false);
            //state.fov_data.set_transparent(x as usize, 19 as usize, false);
        }
        for y in 0..20 {
            let mut idx = map.xy_idx(0, y); 
            map.tiles[idx] = Cell::Wall as u8;
            idx = map.xy_idx(19, y);
            map.tiles[idx] = Cell::Wall as u8;
            //mark 'em as opaque
            //state.fov_data.set_transparent(0 as usize, y as usize, false);
            //state.fov_data.set_transparent(19 as usize, y as usize, false);
        }

        map
    }

}