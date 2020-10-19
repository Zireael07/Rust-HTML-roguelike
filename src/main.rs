extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//the websys Canvas bindings uses it
use wasm_bindgen::JsCast; // for dyn_into
use std::f64;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Player = 0,
    Floor = 1,
    Wall = 2,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
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



/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let mut state = Universe{width:20, height:20,
            cells: vec![Cell::Floor; 20 * 20],
        };
    
        // Make the boundaries walls
        for x in 0..20 {
            state.cells[xy_idx(x, 0)] = Cell::Wall;
            state.cells[xy_idx(x, 19)] = Cell::Wall;
        }
        for y in 0..20 {
            state.cells[xy_idx(0, y)] = Cell::Wall;
            state.cells[xy_idx(19, y)] = Cell::Wall;
        }
    
        //Player
        state.cells[xy_idx(1,1)] = Cell::Player;

        // We'll return the state with the short-hand
        state
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
}

pub fn main() {
    let gs = Universe::new();
    //debug
    log!("We have a universe");
}


// Auto-starts on page load
//start section of the executable may not literally point to main
#[wasm_bindgen(start)]
pub fn start() {
   main()
} 
