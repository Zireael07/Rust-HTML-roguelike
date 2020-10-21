extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//the websys Canvas bindings uses it
use wasm_bindgen::JsCast; // for dyn_into
use std::f64;

//better panics
extern crate console_error_panic_hook;
use std::panic;

//our stuff
mod fov;
use fov::*;


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
    Floor = 0,
    Wall = 1,
}

#[wasm_bindgen]
pub enum Command {
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
}


#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    tiles: Vec<u8>, //Vec<u8> can be passed by wasm_bindgen
    player_position: usize,
    fov: FovRecursiveShadowCasting,
    fov_data: MapData,
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
            tiles: vec![Cell::Floor as u8; 20 * 20],
            player_position: xy_idx(1, 1),
            fov: FovRecursiveShadowCasting::new(),
            fov_data: MapData::new(20,20)
        };
    
        // Make the boundaries walls
        for x in 0..20 {
            state.tiles[xy_idx(x, 0)] = Cell::Wall as u8;
            state.tiles[xy_idx(x, 19)] = Cell::Wall as u8;
            //mark 'em as opaque
            state.fov_data.set_transparent(x as usize, 0 as usize, false);
            state.fov_data.set_transparent(x as usize, 19 as usize, false);
        }
        for y in 0..20 {
            state.tiles[xy_idx(0, y)] = Cell::Wall as u8;
            state.tiles[xy_idx(19, y)] = Cell::Wall as u8;
            //mark 'em as opaque
            state.fov_data.set_transparent(0 as usize, y as usize, false);
            state.fov_data.set_transparent(19 as usize, y as usize, false);
        }
    
        //Player
        //let idx = xy_idx(1, 1);
        //state.player_position = idx;
        
        //let mut fov = FovRecursiveShadowCasting::new();
        state.fov_data.clear_fov(); // compute_fov does not clear the existing fov
        state.fov.compute_fov(&mut state.fov_data, 1, 1, 6, true);
        


        //debug
        log!("We have a universe");

        // We'll return the state with the short-hand
        state
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_tiles(&self) -> Vec<u8> {
        self.tiles.clone()
    }

    // pub fn get_cells_ptr(&self) -> *const Cell {
    //     self.cells.as_ptr()
    // }

    pub fn player(&self) -> Vec<i32> {
        let pos = idx_xy(self.player_position);
        vec![pos.0, pos.1]
    }

    pub fn is_visible(&self, x: usize, y:usize) -> bool {
        return self.fov_data.is_in_fov(x,y);
    }

    pub fn process(&mut self, input: Option<Command>) {
        // New: handle keyboard inputs.
        match input {
            None => {} // Nothing happened
            Some(input) => {
                // A key is pressed or held
                match input {
                    // We're matching a command from the host
                    // and applying movement via the move_player function.

                    // Cursors
                    Command::MoveUp => self.move_player(0, -1),
                    Command::MoveDown => self.move_player(0, 1),
                    Command::MoveLeft => self.move_player(-1, 0),
                    Command::MoveRight => self.move_player(1, 0),

                    _ => {} // Ignore all the other possibilities
                }
            }
        }
    }

    // Handle player movement. Delta X and Y are the relative move
    // requested by the player. We calculate the new coordinates,
    // and if it is a floor - move the player there.
    pub fn move_player(&mut self, delta_x: i32, delta_y: i32) {
        let current_position = idx_xy(self.player_position);
        let new_position = (current_position.0 + delta_x, current_position.1 + delta_y);
        let new_idx = xy_idx(new_position.0, new_position.1);
        if self.tiles[new_idx] == Cell::Floor as u8 {
            self.player_position = new_idx;
            //refresh fov
            self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
            self.fov.compute_fov(&mut self.fov_data, new_position.0 as usize, new_position.1 as usize, 6, true);
        }
    }

}

pub fn main() {
    //let gs = Universe::new();
}


// Auto-starts on page load
//start section of the executable may not literally point to main
#[wasm_bindgen(start)]
pub fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
   //main()
} 
