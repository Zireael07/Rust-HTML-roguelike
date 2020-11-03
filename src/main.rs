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

//RNG
use rand::Rng;

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
    log!("{}", &format!("No path found sx {} sy {} tx {} ty {}", x, y, idx_xy(player_position).0, idx_xy(player_position).1));
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
        let a = state.ecs_world.spawn((Point{x:4, y:4}, Renderable::Thug as u8, "Thug", AI{}));

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
                //enemy turn
                self.get_AI();
            }
            else {
                log!("{}", &format!("Player kicked at the AI"));
                self.attack();
                //enemy turn
                self.get_AI();
            }
        }
    }

    pub fn draw_entities(&self) -> Vec<u8> {
        // Each "drawn" will store 3 u8 values (x,y and tile)
        // based on https://aimlesslygoingforward.com/blog/2017/12/25/dose-response-ported-to-webassembly/ 
        let mut js_drawn = Vec::new();
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &u8)>().iter() {
            if self.is_visible(point.x as usize, point.y as usize) {
                js_drawn.push(point.x as u8);
                js_drawn.push(point.y as u8);
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
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &u8)>().iter() {
            if point.x as usize == x && point.y as usize == y {
                blocked = true;
                break;
            }
        }
        return blocked;
    }

    //a very simple test, akin to flipping a coin or throwing a d2
    fn make_test_d2(&self, skill: u32) -> Vec<bool> {
        let mut rolls = Vec::new();
        for _ in 0..20-skill { // exclusive of end
            rolls.push(rand::random()) // generates a boolean
        }
        return rolls
    }

    fn attack(&self) {
        let res = self.make_test_d2(1);
        log!("{}", format!("Test: {:?}", res));
    }

    
    pub fn get_AI(&mut self) {
        // we need to borrow mutably (for the movement to happen), so we have to use a Point instead of two usizes (hecs limitation)
        for (id, (ai, point)) in &mut self.ecs_world.query::<(&AI, &mut Point)>()
        .with::<&str>() //we can't query it directly above because str length is unknown at compile time
        .iter()
         {
            log!("{}", &format!("Got AI {} x {} y {}",  point.x, point.y, self.ecs_world.get::<&str>(id).unwrap().to_string())); //just unwrapping isn't enough to format
            //if the player's immediately next to us, don't run costly A*
            let player_pos = idx_xy(self.player_position);
            //log!("{}", &format!("Player pos x {} y {}", player_pos.0, player_pos.1));
            if distance2d_chessboard(point.x, player_pos.0, point.y, player_pos.1) < 2 {
                //log!("{}", &format!("AI next to player, attack!"));
                log!("{}", &format!("AI {} kicked at the player", self.ecs_world.get::<&str>(id).unwrap().to_string()));
                self.attack();
            } else {
                let new_pos = path_to_player(&mut self.map, point.x as usize, point.y as usize, self.player_position);
                // move or attack            
                if new_pos.0 == player_pos.0 as usize && new_pos.1 == player_pos.1 as usize {
                    log!("{}", &format!("AI {} kicked at the player", self.ecs_world.get::<&str>(id).unwrap().to_string()));
                    self.attack();
                } else {
                    //actually move
                    point.x = new_pos.0 as i32;
                    point.y = new_pos.1 as i32;
                    //log!("{}", &format!("AI post move x {} y {}",  point.x, point.y));
                }
            }

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
