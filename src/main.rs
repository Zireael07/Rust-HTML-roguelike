extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//the websys Canvas bindings uses it
use wasm_bindgen::JsCast; // for dyn_into
use std::f64;

//better panics
extern crate console_error_panic_hook;
use std::panic;

//ECS
use hecs::World;

//our stuff
mod map;
use map::*;
mod fov;
use fov::*;
mod astar;
use astar::*;
mod utils;
use utils::*;


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
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderable {
    Thug = 0,
}

pub struct AI {}

#[wasm_bindgen]
pub enum Command {
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
}


#[wasm_bindgen]
pub struct Universe {
    map: Map,
    tiles: Vec<u8>, //Vec<u8> can be passed by wasm_bindgen
    player_position: usize,
    fov: FovRecursiveShadowCasting,
    fov_data: MapData,
    ecs_world: World,
}

//it's outside Universe because we're careful not to pass 'self' to it
pub fn path_to_player(map: &mut Map, x: usize, y: usize, player_position: usize) -> (usize, usize) {
    //call A*
    let path = a_star_search(xy_idx(x as i32, y as i32) as i32, player_position as i32, &map);
    if path.success {
        let idx = path.steps[1];
        let idx_pos = idx_xy(idx as usize);
        if !map.is_tile_blocked(idx) {
            let old_idx = (y * map.width as usize) + x;
            //mark as blocked for pathfinding
            map.clear_tile_blocked(old_idx as i32);
            map.set_tile_blocked(idx as i32);
            log!("{}", &format!("Path step x {} y {}", idx_pos.0, idx_pos.1));
            return (idx_pos.0 as usize, idx_pos.1 as usize);
        }
    }
    (x,y) //dummy
}


/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let mut state = Universe{
            map: Map::new(20,20), //{width:20, height:20},
            tiles: vec![Cell::Floor as u8; 20 * 20],
            player_position: xy_idx(1, 1),
            fov: FovRecursiveShadowCasting::new(),
            fov_data: MapData::new(20,20),
            ecs_world: World::new(),
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
        
        //spawn entity
        let a = state.ecs_world.spawn((4 as usize, 4 as usize, Renderable::Thug as u8, "Thug", AI{}));

        //debug
        log!("We have a universe");

        // We'll return the state with the short-hand
        state
    }

    pub fn width(&self) -> u32 {
        self.map.width
    }

    pub fn height(&self) -> u32 {
        self.map.height
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
            if !self.blocking_creatures_at(new_position.0 as usize, new_position.1 as usize) {
                self.player_position = new_idx;
                //refresh fov
                self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
                self.fov.compute_fov(&mut self.fov_data, new_position.0 as usize, new_position.1 as usize, 6, true);
                self.get_AI();
            }
        }
    }

    pub fn draw_entities(&self) -> Vec<u8> {
        // Each "drawn" will store 3 u8 values (x,y and tile)
        // based on https://aimlesslygoingforward.com/blog/2017/12/25/dose-response-ported-to-webassembly/ 
        let mut js_drawn = Vec::new();
        for (id, (pos_x, pos_y, render)) in self.ecs_world.query::<(&usize, &usize, &u8)>().iter() {
            if self.is_visible(*pos_x, *pos_y) {
                js_drawn.push(*pos_x as u8);
                js_drawn.push(*pos_y as u8);
                js_drawn.push(*render);
                //log!("{}", &format!("Rust: x {} y {} tile {}", pos_x, pos_y, render));
            }
        }

        return js_drawn;
    }

}

//Methods not exposed to JS
impl Universe {
    pub fn blocking_creatures_at(&self, x: usize, y: usize) -> bool {
        let mut blocked = false;
        for (id, (pos_x, pos_y, render)) in self.ecs_world.query::<(&usize, &usize, &u8)>().iter() {
            if *pos_x == x && *pos_y == y {
                blocked = true;
                break;
            }
        }
        return blocked;
    }
    
    pub fn get_AI(&mut self) {
        for (id, (ai, pos_x, pos_y)) in self.ecs_world.query::<(&AI, &usize, &usize)>()
        .with::<&str>() //we can't query it directly above because str length is unknown at compile time
        .iter()
         {
            log!("{}", &format!("Got AI {} x {} y {}",  pos_x, pos_y, self.ecs_world.get::<&str>(id).unwrap().to_string())); //just unwrapping isn't enough to format
            let new_position = path_to_player(&mut self.map, *pos_x, *pos_y, self.player_position);
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
