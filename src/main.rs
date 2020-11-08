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
use hecs::Entity;

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

//using web_sys here because I am not too sure on how to pass strings to custom JS
pub fn game_message(string: &str)
{
    let window = web_sys::window().expect("global window does not exists");    
    let document = window.document().expect("expecting a document on window");
    
    let messages = document.get_element_by_id("messages").unwrap();
    let line = document.create_element("div").unwrap();
    line.set_inner_html(string);
    messages.append_child(&line).unwrap(); //implicitly converts to Node

    //axe the first if more than 5
    while messages.child_element_count() > 5 {
        messages.remove_child(&messages.first_element_child().unwrap()); //implicit conversion
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
    Knife = 1,
}

//for ECS
pub struct Player{}
pub struct AI {}
pub struct Item{}
pub struct InBackpack{}
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}

//input
#[wasm_bindgen]
pub enum Command {
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
    GetItem,
    Inventory,
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
        
        //rendering and position handled otherwise, so the player Entity only needs combat stats
        let player = state.ecs_world.spawn((Player{}, CombatStats{hp:20, max_hp: 20, defense:1, power:1}));

        //spawn entity
        let a = state.ecs_world.spawn((Point{x:4, y:4}, Renderable::Thug as u8, "Thug", AI{}, CombatStats{hp:10, max_hp:10, defense:1, power:1}));
        let it = state.ecs_world.spawn((Point{x:6,y:7}, Renderable::Knife as u8, "Combat knife", Item{}));

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

                    //others
                    Command::GetItem => self.get_item(),

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
            let blocker = self.blocking_creatures_at(new_position.0 as usize, new_position.1 as usize);

            match blocker {
                Some(entity) => { 
                    //this assumes the blocker has a name!
                    game_message(&format!("Player kicked the {}", self.ecs_world.get::<&str>(entity).unwrap().to_string()));
                    self.attack(&entity);
                    //enemy turn
                    self.get_AI();
                    self.remove_dead();
                },
                None => {
                    self.player_position = new_idx;
                    //refresh fov
                    self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
                    self.fov.compute_fov(&mut self.fov_data, new_position.0 as usize, new_position.1 as usize, 6, true);
                    //enemy turn
                    self.get_AI();
                    self.remove_dead();
                }
            }
                 
        }
    }

    pub fn get_item(&mut self) {
        let current_position = idx_xy(self.player_position);
        let item = self.items_at(current_position.0 as usize, current_position.1 as usize);

        match item {
            Some(entity) => {
                //this assumes the blocker has a name!
                game_message(&format!("Player picked up {}", self.ecs_world.get::<&str>(entity).unwrap().to_string()));
                //puts the item in backpack
                self.pickup_item(&entity)
            },
            None => { 
                game_message(&format!("No item to pick up here"));
            },
        }
    }

    pub fn draw_entities(&self) -> Vec<u8> {
        // Each "drawn" will store 3 u8 values (x,y and tile)
        // based on https://aimlesslygoingforward.com/blog/2017/12/25/dose-response-ported-to-webassembly/ 
        let mut js_drawn = Vec::new();
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &u8)>()
        .without::<InBackpack>() //no ref/pointer here!
        .iter() {
            if self.is_visible(point.x as usize, point.y as usize) {
                js_drawn.push(point.x as u8);
                js_drawn.push(point.y as u8);
                js_drawn.push(*render);
                //log!("{}", &format!("Rust: x {} y {} tile {}", point.x, point.y, render));
            }
        }

        return js_drawn;
    }


    pub fn inventory_size(&self) -> usize {
        return self.items_in_inventory().len()
    }

    pub fn inventory_items(&self) -> Vec<u64> {
        return self.items_in_inventory();
    }

    //unfortunately we can't pass a Vec<&str> to JS... nor borrowed refs like &str
    // so instead, we store a list of ids and get the actual strings with this separate function
    pub fn inventory_name_for_id(&self, id: u64) -> String {
        //let item = self.ecs_world.find_entity_from_id(id); //not present in hecs 0.2.15
        let item = hecs::Entity::from_bits(id); //restore
        return self.ecs_world.get::<&str>(item).unwrap().to_string()
    }

}

//Methods not exposed to JS
impl Universe {
    pub fn blocking_creatures_at(&self, x: usize, y: usize) -> Option<Entity> {
        let mut blocked: Option<Entity> = None;
        for (id, (point, combat)) in self.ecs_world.query::<(&Point, &CombatStats)>().iter() {
            if point.x as usize == x && point.y as usize == y {
                blocked = Some(id);
                break;
            }
        }
        return blocked;
    }

    pub fn items_at(&self, x: usize, y: usize) -> Option<Entity> {
        let mut item: Option<Entity> = None;
        for (id, (point, it)) in self.ecs_world.query::<(&Point, &Item)>()
        .without::<InBackpack>() //no ref/pointer here!!!
        .iter() {
            if point.x as usize == x && point.y as usize == y {
                item = Some(id);
                break;
            }
        }
        return item;
    }

    pub fn items_in_inventory(&self) -> Vec<u64>{
        let mut ids = Vec::new();
        //test
        for (id, (item, backpack)) in &mut self.ecs_world.query::<(&Item, &InBackpack)>().iter(){
            log!("{}", &format!("Item in inventory: {}", self.ecs_world.get::<&str>(id).unwrap().to_string()));
            //log!("{}", &format!("ID: {:?}", id));
            ids.push(id.to_bits()); //we can't get from id later on, yet
        }
        return ids;
    }

    pub fn pickup_item(&mut self, item: &Entity) {
        self.ecs_world.insert_one(*item, InBackpack{});
        self.items_in_inventory();
    }


    //a very simple test, akin to flipping a coin or throwing a d2
    fn make_test_d2(&self, skill: u32) -> Vec<bool> {
        let mut rolls = Vec::new();
        for _ in 0..20-skill { // exclusive of end
            rolls.push(rand::random()) // generates a boolean
        }
        return rolls
    }

    fn attack(&self, target: &Entity) {
        let res = self.make_test_d2(1);
        let sum = res.iter().filter(|&&b| b).count(); //iter returns references and filter works with references too - double indirection
        log!("{}", &format!("Test: {:?} sum: {}", res, sum));
        //deal damage
        // the mut here is obligatory!!!
        let mut stats = self.ecs_world.get_mut::<CombatStats>(*target).unwrap();
        stats.hp = stats.hp - 2;
        //if killed, despawn
        //borrow checker doesn't allow this??
        // if stats.hp <= 0 {
        //     self.ecs_world.despawn(*target).unwrap();
        //     log!("{}", &format!("Target was killed!"));
        // }
    }

    fn remove_dead(&mut self) {
        // Here we query entities with 0 or less hp and despawn them
        let mut to_remove: Vec<Entity> = Vec::new();
        for (id, stats) in &mut self.ecs_world.query::<&CombatStats>() {
            if stats.hp <= 0 {
                to_remove.push(id);
            }
        }

        for entity in to_remove {
            game_message(&format!("AI {} is dead", self.ecs_world.get::<&str>(entity).unwrap().to_string()));
            self.ecs_world.despawn(entity).unwrap();
        }
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
                game_message(&format!("AI {} kicked at the player", self.ecs_world.get::<&str>(id).unwrap().to_string()));
                //get player entity
                let mut play: Option<Entity> = None;
                for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
                    play = Some(id);
                }
                match play {
                    Some(entity) => self.attack(&entity),
                    None => {},
                }
                
            } else {
                let new_pos = path_to_player(&mut self.map, point.x as usize, point.y as usize, self.player_position);
                // move or attack            
                if new_pos.0 == player_pos.0 as usize && new_pos.1 == player_pos.1 as usize {
                    game_message(&format!("AI {} kicked at the player", self.ecs_world.get::<&str>(id).unwrap().to_string()));
                    //get player entity
                    let mut play: Option<Entity> = None;
                    for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
                        play = Some(id);
                    }
                    match play {
                        Some(entity) => self.attack(&entity),
                        None => {},
                    }

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
