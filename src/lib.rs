// those work on module level, too - https://stackoverflow.com/a/39269962
#![allow(unused_must_use)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//the websys bindings uses it
use wasm_bindgen::JsCast; // for dyn_into

//better panics
extern crate console_error_panic_hook;
use std::panic;

//ECS
use hecs::World;
use hecs::Entity;

//RNG
use rand::Rng;

//save/load
use serde::{Serialize, Deserialize};
use serde_json::json;

use std::fmt;

//our stuff

//3rd party vendored in
mod fastnoise;

mod map;
use map::*;
mod fov;
use fov::*;
mod astar;
use astar::*;
mod utils;
use utils::*;
mod rect;
use rect::*;
mod map_builders;
use map_builders::*;

//lisp-y
mod lispy;
use lispy::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

//TODO: shuffle all or most of this to JS because Rust is clunky when it comes to DOM... :/
//using web_sys here because I am not too sure on how to pass strings to custom JS
pub fn game_message(string: &str)
{
    //convert
    let mut string = string.to_string();
    let window = web_sys::window().expect("global window does not exists");    
    let document = window.document().expect("expecting a document on window");
    
    let messages = document.get_element_by_id("messages").unwrap();
    let line = document.create_element("div").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap(); //dyn_into for style() to work

    //apply CSS to whole line
    if string.starts_with("{r"){
        //strip tag
        string = string.trim_start_matches("{r").to_string();
        line.style().set_property("color", "rgb(255,0,0)");
    }
    if string.starts_with("{gr"){
        //strip tag
        string = string.trim_start_matches("{gr").to_string();
        line.style().set_property("color", "rgb(127,127,127)");
    }
    if string.starts_with("{g"){
        //strip tag
        string = string.trim_start_matches("{g").to_string();
        line.style().set_property("color", "rgb(0,255,0)");
    }

    //detect in-line styles
    if string.contains("{r") {
        //parse style
        string = string.replace("{r", "<span style=\"color:rgb(255,0,0)\">");
        string = string.replace("}", "</span>");
    }
    if string.contains("{gr") {
        //parse style
        string = string.replace("{gr", "<span style=\"color:rgb(127,127,127)\">");
        string = string.replace("}", "</span>");
    }
    if string.contains("{g") {
        //parse style
        string = string.replace("{g", "<span style=\"color:rgb(0,255,0)\">");
        string = string.replace("}", "</span>");
    }
    if string.contains("{c"){
        //parse style
        string = string.replace("{c", "<span style=\"color:rgb(0,255,255)\">");
        string = string.replace("}", "</span>");
    }
    if string.contains("{y"){
        //parse style
        string = string.replace("{y", "<span style=\"color:rgb(255,255,0)\">");
        string = string.replace("}", "</span>");
    }

    if string.contains("🇪 🇸"){
        //parse country flag
        string = string.replace("🇪 🇸", "<img src=\"./es.svg\" alt=\"\" style=\"height:14px\">");
    }


    line.set_inner_html(&string); //wants &str
    messages.append_child(&line).unwrap(); //implicitly converts to Node

    //axe the first if more than 5
    while messages.child_element_count() > 5 {
        messages.remove_child(&messages.first_element_child().unwrap()); //implicit conversion
    }
}	


#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Renderable {
    Thug = 0,
    Knife = 1,
    Medkit = 2,
    Barkeep = 3,
    Table = 4,
    Chair = 5,
    Boots = 6,
    Jacket = 7,
    Jeans = 8
}

//for ECS
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Player{}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Needs{
    pub hunger: i32,
    pub thirst: i32,
}
pub struct Path{
    pub steps: Vec<i32> // see astar line 43
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub base : i32, // equal to what would've been the modifier in d20
    pub bonus : i32
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Attributes {
    pub strength : Attribute,
    pub dexterity : Attribute,
    pub constitution : Attribute,
    pub intelligence : Attribute,
    pub wisdom : Attribute,
    pub charisma : Attribute,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct AI {}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Money {
    pub money: f32
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum FactionType { Enemy, Townsfolk }

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Faction {
    pub typ: FactionType
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vendor {
    //pub categories : Vec<String>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Item{}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InBackpack{}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Consumable{} //in the sense that it is limited use only
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProvidesFood {}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProvidesQuench {}


//don't need to be serialized
pub struct WantsToUseItem {
    pub item : Entity
}
// tells the engine to nuke us
pub struct ToRemove {pub yes: bool} //bool is temporary while we can't modify entities when iterating

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee, Torso, Legs, Feet }
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Equippable {
    pub slot : EquipmentSlot
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Equipped {
    pub owner : u64, //because Entity cannot be serialized by serde
    pub slot : EquipmentSlot
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MeleeBonus {
    pub bonus : i32
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DefenseBonus {
    pub bonus : f32
}

//make a struct so that....
pub struct Rolls(Vec<bool>);

// .. we can implement `Display`

//ref: https://stackoverflow.com/questions/54042984/can-i-format-debug-output-as-binary-when-the-values-are-in-a-vector 
//ref: https://doc.rust-lang.org/rust-by-example/hello/print/print_display.html
impl fmt::Display for Rolls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // extract the value using tuple idexing
        // and create reference to 'vec'
        let vec = &self.0;

        // @count -> the index of the value,
        // @n     -> the value
        for (count, n) in vec.iter().enumerate() { 
            if count != 0 { write!(f, " ")?; }

            if n == &false {
                write!(f, "{{gr0}}")?; //this is format, so we need to escape
            } else {
                write!(f, "{{c1}}")?;
            }

        }
        Ok(())
    }
}



// what it says on the tin
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    entity: u64, //because Entity cannot be serialized by serde
    name: String,
    point: Option<Point>,
    render: Option<u8>,
    player: Option<Player>,
    needs: Option<Needs>,
    money: Option<Money>,
    ai: Option<AI>,
    vendor: Option<Vendor>,
    combat: Option<CombatStats>,
    faction: Option<Faction>,
    item: Option<Item>,
    backpack: Option<InBackpack>,
    consumable: Option<Consumable>,
    heals: Option<ProvidesHealing>,
    food: Option<ProvidesFood>,
    quench: Option<ProvidesQuench>,
    equippable: Option<Equippable>,
    meleebonus: Option<MeleeBonus>,
    equip: Option<Equipped>,
}


//input
#[wasm_bindgen]
pub enum Command {
//    None = -1, //unfortunately we can't use -1 in wasm_bindgen...
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
    GetItem,
    Inventory,
    SaveGame,
}


#[wasm_bindgen]
pub struct Universe {
    map: Map,
    player_position: usize,
    fov: FovRecursiveShadowCasting,
    fov_data: MapData,
    ecs_world: World,
}

//it's outside Universe because we're careful not to pass 'self' to it
pub fn path_to_player(map: &mut Map, x: usize, y: usize, player_position: usize) -> (usize, usize) {
    //call A*
    let path = a_star_search(map.xy_idx(x as i32, y as i32) as i32, player_position as i32, &map);
    if path.success {
        let idx = path.steps[1];
        let idx_pos = map.idx_xy(idx as usize);
        if !map.is_tile_blocked(idx) {
            let old_idx = (y * map.width as usize) + x;
            //mark as blocked for pathfinding
            map.clear_tile_blocked(old_idx as i32);
            map.set_tile_blocked(idx as i32);
            log!("{}", &format!("Path step x {} y {}", idx_pos.0, idx_pos.1));
            return (idx_pos.0 as usize, idx_pos.1 as usize);
        }
    }
    log!("{}", &format!("No path found sx {} sy {} tx {} ty {}", x, y, map.idx_xy(player_position).0, map.idx_xy(player_position).1));
    (x,y) //dummy
}

pub fn player_path_to_target(map: &mut Map, player_position: usize, x: usize, y: usize) -> Vec<i32> {
    let path = a_star_search(player_position as i32, map.xy_idx(x as i32, y as i32) as i32, &map);
    if path.success {
        return path.steps;
    }
    log!("{}", &format!("No player path found, x {} y {}", x,y));
    vec![player_position as i32] //dummy
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let mut state = Universe{
            map: Map::new(2,2), //dummy
            player_position: 0, //dummy
            fov: FovRecursiveShadowCasting::new(),
            fov_data: MapData::new(2,2), //dummy
            ecs_world: World::new(),
        };
    
        state.player_position = state.map.xy_idx(1,1); //default

        //mapgen
        let mut builder = map_builders::random_builder(80,60);
        builder.build_map();
        state.map = builder.build_data.map.clone();

        //spawn
        match builder.build_data.starting_position {
            None => {},
            Some(point) => {
                state.player_position = state.map.xy_idx(point.x, point.y);
            }
        }


        state.fov_data = MapData::new(80,60);

        //build FOV cache
        for (idx, tile) in state.map.tiles.iter().enumerate() {
            if *tile == Cell::Wall as u8 {
                state.fov_data.set_transparent(state.map.idx_xy(idx).0 as usize, state.map.idx_xy(idx).1 as usize, false);
            }
        }
    
        state.fov_data.clear_fov(); // compute_fov does not clear the existing fov
        state.fov.compute_fov(&mut state.fov_data, state.map.idx_xy(state.player_position).0 as usize, state.map.idx_xy(state.player_position).1 as usize, 6, true);
        //reveal tiles
        for (idx, b) in state.fov_data.fov.iter().enumerate() {
            if *b {
                state.map.revealed_tiles[idx] = true;
            }
        }
        
        //rendering and position handled otherwise, so the player Entity only needs combat stats
        //NOTE: player is always entity id 0
        // 15, 14, 13, 12, 10, 8 aka elite array
        let player = state.ecs_world.spawn(("Player".to_string(), Player{}, CombatStats{hp:20, max_hp: 20, defense:1, power:1}, Money{money:100.0}, Needs{hunger:500, thirst:300}, 
        Attributes{strength:Attribute{base:2, bonus:0}, dexterity:Attribute{base:1, bonus:0}, constitution:Attribute{base:2, bonus:0}, intelligence:Attribute{base:1,bonus:0}, wisdom:Attribute{base:-1,bonus:0}, charisma:Attribute{base:0,bonus:0}}));
        //starting inventory
        state.give_item("Protein shake".to_string());
        state.give_item("Medkit".to_string());


        //spawn anything listed
        state.spawn_entities(builder.build_data.list_spawns);

        //spawn entities
        let th = state.ecs_world.spawn((Point{x:4, y:4}, Renderable::Thug as u8, "Thug".to_string(), AI{}, Faction{typ: FactionType::Enemy}, CombatStats{hp:10, max_hp:10, defense:1, power:1}));
        //their starting equipment
        let boots = state.ecs_world.spawn((Point{x:4, y:4}, Renderable::Boots as u8, "Boots".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Feet }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let l_jacket = state.ecs_world.spawn((Point{x:4,y:4}, Renderable::Jacket as u8, "Leather jacket".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Torso }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let jeans = state.ecs_world.spawn((Point{x:4,y:4}, Renderable::Jeans as u8, "Jeans".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Legs}, DefenseBonus{ bonus:0.1}, ToRemove{yes:false}));
        state.ecs_world.insert_one(boots, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Feet});
        state.ecs_world.insert_one(l_jacket, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Torso});
        state.ecs_world.insert_one(jeans, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Legs});

        let it = state.ecs_world.spawn((Point{x:6,y:7}, Renderable::Knife as u8, "Combat knife".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Melee }, MeleeBonus{ bonus: 2}, ToRemove{yes:false}));
        let med = state.ecs_world.spawn((Point{x:5, y:5}, Renderable::Medkit as u8, "Medkit".to_string(), Item{}, ToRemove{yes:false}, Consumable{}, ProvidesHealing{heal_amount:5}));
        let boots = state.ecs_world.spawn((Point{x:6, y:18}, Renderable::Boots as u8, "Boots".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Feet }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let l_jacket = state.ecs_world.spawn((Point{x:6,y:18}, Renderable::Jacket as u8, "Leather jacket".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Torso }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let jeans = state.ecs_world.spawn((Point{x:6,y:18}, Renderable::Jeans as u8, "Jeans".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Legs}, DefenseBonus{ bonus:0.1}, ToRemove{yes:false}));
        
        
        
        //let b = state.ecs_world.spawn((Point{x:6, y: 18}, Renderable::Barkeep as u8, "Barkeep".to_string(), Faction{typ: FactionType::Townsfolk}, CombatStats{hp:5, max_hp:5, defense:1, power:1}));

        //debug
        //log!("{}", &format!("Player stats: {:?}", *state.ecs_world.get::<Attributes>(player).unwrap()));
       
        log!("We have a universe");

        //lispy test
        parse_script();

        // We'll return the state with the short-hand
        state
    }

    pub fn on_game_start(&self) {
        game_message(&format!("{{cWelcome to Neon Twilight!"));
    }

    pub fn width(&self) -> u32 {
        self.map.width
    }

    pub fn height(&self) -> u32 {
        self.map.height
    }

    pub fn get_tiles(&self) -> Vec<u8> {
        self.map.tiles.clone()
    }


    pub fn player(&self) -> Vec<i32> {
        let pos = self.map.idx_xy(self.player_position);
        vec![pos.0, pos.1]
    }

    pub fn is_visible(&self, x: usize, y:usize) -> bool {
        return self.fov_data.is_in_fov(x,y);
    }

    pub fn is_seen(&self, x: usize, y:usize) -> bool {
        return self.map.revealed_tiles[self.map.xy_idx(x as i32, y as i32)];
    }

    pub fn should_draw(&self, x: usize, y:usize) -> bool {
        return self.is_visible(x,y) || self.is_seen(x,y);
    }

    pub fn spawn(&mut self, x:i32, y:i32, name:String) {
        //TODO: should be a dict lookup
        // props
        if name == "Table".to_string() {
            self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Table as u8));
        } else if name == "Chair".to_string() {
            self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Chair as u8));
        }
        //NPCs
        else if name == "Barkeep".to_string() {
            self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Barkeep as u8, "Barkeep".to_string(), Faction{typ: FactionType::Townsfolk}, CombatStats{hp:5, max_hp:5, defense:1, power:1}, Vendor{}));
        } else {
            let th = self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Thug as u8, "Thug".to_string(), AI{}, Faction{typ: FactionType::Enemy}, CombatStats{hp:10, max_hp:10, defense:1, power:1}));
            //their starting equipment
            let boots = self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Boots as u8, "Boots".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Feet }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
            let l_jacket = self.ecs_world.spawn((Point{x:x,y:y}, Renderable::Jacket as u8, "Leather jacket".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Torso }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
            let jeans = self.ecs_world.spawn((Point{x:x,y:y}, Renderable::Jeans as u8, "Jeans".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Legs}, DefenseBonus{ bonus:0.1}, ToRemove{yes:false}));
            self.ecs_world.insert_one(boots, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Feet});
            self.ecs_world.insert_one(l_jacket, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Torso});
            self.ecs_world.insert_one(jeans, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Legs});
        }
    }

    pub fn process(&mut self, input: Option<Command>) {
        // New: handle keyboard inputs.
        match input {
            None => {}, // Nothing happened
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

                    //save/load
                    //Command::SaveGame => self.save_game(),

                    _ => {} // Ignore all the other possibilities
                }
            }
        }
    }

    pub fn astar_path(&mut self, x:i32, y:i32) {
        if self.is_player_dead() {
            return;
        }
        let new_path = player_path_to_target(&mut self.map,  self.player_position, x as usize, y as usize);

        //debugging
        for i in &new_path {
            log!("{}", &format!("x {} y {}", self.map.idx_xy(*i as usize).0, self.map.idx_xy(*i as usize).1));
        }

        self.set_automove(new_path);

    }

    // Handle player movement. Delta X and Y are the relative move
    // requested by the player. We calculate the new coordinates,
    // and if it is a floor - move the player there.
    pub fn move_player(&mut self, delta_x: i32, delta_y: i32) {
        if self.is_player_dead() {
            return;
        }

        let current_position = self.map.idx_xy(self.player_position);
        let new_position = (current_position.0 + delta_x, current_position.1 + delta_y);
        let new_idx = self.map.xy_idx(new_position.0, new_position.1);
        if self.map.is_tile_walkable(new_position.0, new_position.1) {
            let blocker = self.blocking_creatures_at(new_position.0 as usize, new_position.1 as usize);

            match blocker {
                Some(entity) => { 
                    let fact = self.ecs_world.get::<Faction>(entity).unwrap().typ;
                    if fact == FactionType::Enemy {
                            //this assumes the blocker has a name!
                            game_message(&format!("{{gPlayer kicked the {}", self.ecs_world.get::<String>(entity).unwrap().to_string()));
                            self.attack(&entity);
                    } else if fact == FactionType::Townsfolk {
                        if self.ecs_world.get::<Vendor>(entity).is_ok() {
                            //game_message(&format!("You talk to the vendor"));
                            //GUI
                            let window = web_sys::window().expect("global window does not exists");    
                            let document = window.document().expect("expecting a document on window");                        
                            let vendor = document.get_element_by_id("vendor").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
                            let list = vendor.class_list().toggle("visible");
                        } else {
                            game_message(&format!("The man says 🇪 🇸: hola!"));
                        }
                    }

                    //enemy turn
                    self.get_AI();
                    self.remove_dead();
                    self.survival_tick();
                },
                None => {
                    self.player_position = new_idx;
                    //refresh fov
                    self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
                    self.fov.compute_fov(&mut self.fov_data, new_position.0 as usize, new_position.1 as usize, 6, true);
                    //reveal tiles
                    for (idx, b) in self.fov_data.fov.iter().enumerate() {
                        if *b {
                            self.map.revealed_tiles[idx] = true;
                        }
                    }

                    //test MUD-style description
                    let area_desc = "This area appears to be a town that hugs a forest.";
                    let mut terrain_desc = "";
                    if (self.map.tiles[new_idx] == Cell::Grass as u8){
                        terrain_desc = "You feel the grass under your feet.";
                    }
                    else if (self.map.tiles[new_idx] == Cell::Floor as u8) {
                        terrain_desc = " You walk on paved ground of the town.";
                    }
                    else if (self.map.tiles[new_idx] == Cell::FloorIndoor as u8) {
                        terrain_desc = " You entered one of the buildings.";
                    }
                    game_message(&format!("{} {}", area_desc, terrain_desc));


                    //enemy turn
                    self.get_AI();
                    self.remove_dead();
                    self.survival_tick();
                }
            }
                 
        }
        else {
            log!("{}", &format!("Blocked move to {}, {} ", new_position.0,new_position.1))
        }
    }

    pub fn set_automove(&mut self, path: Vec<i32>) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                self.ecs_world.insert_one(entity, Path{ steps: path});
            },
            None => { }
        }
    }

    pub fn has_automove(&self) -> bool {
        if self.is_player_dead() {
            return false;
        }
        else {
            return true;
        }
    }

    pub fn get_automove(&self) -> Vec<i32> {
         //get player entity
         let mut play: Option<Entity> = None;
         for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
             play = Some(id);
         }
         match play {
             Some(entity) => {
                let path = self.ecs_world.get_mut::<Path>(entity);
                if path.is_ok() {
                    let mut steps = path.unwrap().steps.clone();
                    steps.remove(0);
                   return steps;
                } else {
                    return [].to_vec();
                }
             },
             None => { return [].to_vec(); }
            }
    }

    pub fn advance_automove(&mut self) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                self.ecs_world.get_mut::<Path>(entity).unwrap().steps.remove(0);
                if self.ecs_world.get::<Path>(entity).unwrap().steps.len() < 1 {
                    self.ecs_world.remove_one::<Path>(entity);
                }
            },
            None => {}
        }
    }


    pub fn get_item(&mut self) {
        let current_position = self.map.idx_xy(self.player_position);
        let item = self.items_at(current_position.0 as usize, current_position.1 as usize);

        match item {
            Some(entity) => {
                //this assumes the blocker has a name!
                game_message(&format!("Player picked up {}", self.ecs_world.get::<String>(entity).unwrap().to_string()));
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
        .without::<InBackpack>().without::<Equipped>() //no ref/pointer here!
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

    pub fn view_list(&self) -> Vec<u64> {
        let mut list = Vec::new();
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &u8)>()
        .with::<String>()
        .without::<InBackpack>().without::<Equipped>() //no ref/pointer here!
        .iter() {
            if self.is_visible(point.x as usize, point.y as usize) {
                list.push(id.to_bits())
            }
        }
        return list;
    }

    //we store a list of ids and get the actual strings with this separate function
    pub fn view_string_for_id(&self, id: u64) -> String {
        let ent = hecs::Entity::from_bits(id); //restore

        let player_pos = self.map.idx_xy(self.player_position);
        let point = self.ecs_world.get::<Point>(ent).unwrap();
        let direction = dir(&Point{x:player_pos.0, y:player_pos.1}, &Point{x:point.x, y:point.y});
        let dist = distance2d_chessboard(point.x, player_pos.0, point.y, player_pos.1);
        let name = self.ecs_world.get::<String>(ent).unwrap().to_string();
        return format!("{} - {} {:?}", name, dist, direction);
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

        // add (equipped) for items that are, well, equipped
        let mut name = self.ecs_world.get::<String>(item).unwrap().to_string();
        if self.ecs_world.get::<Equipped>(item).is_ok(){
            name = name + " (equipped)" //Rust string concat is easy!
        }
        return name
    }

    pub fn use_item_ext(&mut self, id: u64) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                //check dead
                let hp = self.ecs_world.get::<CombatStats>(entity).unwrap().hp;
                if hp <= 0 {
                    return 
                }

                log!("Player uses item {}", id);
                let item = hecs::Entity::from_bits(id); //restore
                self.use_item(&entity, &item);
                self.remove_dead(); //in case we used a consumable item
            },
            None => {},
        }
    }

    pub fn change_money(&mut self, val: f32) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let mut purse = self.ecs_world.get_mut::<Money>(entity).unwrap();
                purse.money = purse.money - val;
            },
            None => {},
        }
    }

    pub fn give_item(&mut self, name: String) {
        let current_position = self.map.idx_xy(self.player_position);

        let mut item: Option<Entity> = None;
        //TODO: should be a dict lookup
        if name == "Protein shake".to_string() {
            item = Some(self.ecs_world.spawn((Point{x:current_position.0,y:current_position.1}, Renderable::Medkit as u8, "Protein shake".to_string(), Item{}, ProvidesFood{}, ProvidesQuench{}, Consumable{}, ToRemove{yes:false})));
        }
        if name == "Medkit".to_string() {
            item = Some(self.ecs_world.spawn((Point{x:5, y:5}, Renderable::Medkit as u8, "Medkit".to_string(), Item{}, ToRemove{yes:false}, Consumable{}, ProvidesHealing{heal_amount:5})));
        }
        match item {
            Some(it) => {
                //puts the item in backpack
                self.pickup_item(&it);
            },
            None => {},
        }

    }

    pub fn set_player_stats(&mut self, new_stats: Vec<i32>) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let mut stats = self.ecs_world.get_mut::<Attributes>(entity).unwrap();
                stats.strength.base = new_stats[0];
                stats.dexterity.base = new_stats[1];
                stats.constitution.base = new_stats[2];
                stats.intelligence.base = new_stats[3];
                stats.wisdom.base = new_stats[4];
                stats.charisma.base = new_stats[5];

                log!("{}", &format!("Player stats: {:?}", *stats));
            },
            None => {},
        }
    }

    //i32 because that's what JS sends in
    pub fn describe(&self, x:i32, y:i32) -> String {
        let current_position = self.map.idx_xy(self.player_position);

        let ent = self.entities_at(x as usize, y as usize);

        let mut desc = "".to_string();
        match ent {
            Some(entity) => {
                // display name and faction if any 
                desc = self.ecs_world.get::<String>(entity).unwrap().to_string();
                if self.ecs_world.get::<Faction>(entity).is_ok() {
                    desc = desc + &format!("\n{:?}", self.ecs_world.get::<Faction>(entity).unwrap().typ);
                }
            },
            None => { }
        }
        return format!("Direction: {:?}\n {}", dir(&Point{x:current_position.0, y:current_position.1}, &Point{x:x, y:y}), desc);

        //log!("{}", &format!("Direction: {:?}", dir(&Point{x:current_position.0, y:current_position.1}, &Point{x:x, y:y})));
    }

    pub fn get_description(&self, x:i32, y:i32) -> String {
        return self.describe(x,y);
    }

    //save/load
    pub fn save_game(&self) -> String {
        log!("Saving game...");
        //iterate over all entities
        let entities = self.ecs_world.iter().map(|(id, _)| id).collect::<Vec<_>>();
        let mut save_datas : Vec<SaveData> = Vec::new();
    
        for e in entities {
            //note to self: JSON macro doesn't work with conditionals
            //so we need an intermediate struct
            let mut saved = SaveData{
                entity: e.to_bits(),
                point: None,
                render: None,
                name: "".to_string(), //because props don't have names //self.ecs_world.get::<String>(e).unwrap().to_string(),
                player: None,
                needs: None,
                ai: None,
                money: None,
                faction: None,
                vendor: None,
                combat: None,
                item: None,
                backpack: None,
                consumable: None,
                heals: None,
                food: None,
                quench: None,
                equippable: None,
                meleebonus: None,
                equip : None,
            };

            //log!("{:?}", e);

            // player doesn't have point or renderable
            if self.ecs_world.get::<Point>(e).is_ok() {
                saved.point = Some(*self.ecs_world.get::<Point>(e).unwrap()); //they all need to be dereferenced
            }
            if self.ecs_world.get::<u8>(e).is_ok() {
                saved.render = Some(*self.ecs_world.get::<u8>(e).unwrap());
            }
            //props don't have names
            if self.ecs_world.get::<String>(e).is_ok(){
                saved.name = self.ecs_world.get::<String>(e).unwrap().to_string()
            }
            //those aren't guaranteed
            if self.ecs_world.get::<Player>(e).is_ok() {
                //log!("{:?} is player", e);
                saved.player = Some(*self.ecs_world.get::<Player>(e).unwrap());
                //save player position
                let current_position = self.map.idx_xy(self.player_position);
                saved.point = Some(Point{x:current_position.0, y:current_position.1});
            }
            if self.ecs_world.get::<AI>(e).is_ok(){
                saved.ai = Some(*self.ecs_world.get::<AI>(e).unwrap());
            }
            if self.ecs_world.get::<Needs>(e).is_ok(){
                saved.needs = Some(*self.ecs_world.get::<Needs>(e).unwrap());
            }
            if self.ecs_world.get::<Money>(e).is_ok(){
                saved.money = Some(*self.ecs_world.get::<Money>(e).unwrap());
            }
            if self.ecs_world.get::<Faction>(e).is_ok(){
                saved.faction = Some(*self.ecs_world.get::<Faction>(e).unwrap());
            }
            if self.ecs_world.get::<Vendor>(e).is_ok(){
                saved.vendor = Some(*self.ecs_world.get::<Vendor>(e).unwrap());
            }
            if self.ecs_world.get::<CombatStats>(e).is_ok(){
                saved.combat = Some(*self.ecs_world.get::<CombatStats>(e).unwrap());
            }
            if self.ecs_world.get::<Item>(e).is_ok() {
                saved.item = Some(*self.ecs_world.get::<Item>(e).unwrap());
            }
            if self.ecs_world.get::<InBackpack>(e).is_ok() {
                saved.backpack = Some(*self.ecs_world.get::<InBackpack>(e).unwrap());
            }
            if self.ecs_world.get::<Consumable>(e).is_ok() {
                saved.consumable = Some(*self.ecs_world.get::<Consumable>(e).unwrap());
            }
            if self.ecs_world.get::<ProvidesHealing>(e).is_ok(){
                saved.heals = Some(*self.ecs_world.get::<ProvidesHealing>(e).unwrap());
            }
            if self.ecs_world.get::<ProvidesFood>(e).is_ok(){
                saved.food = Some(*self.ecs_world.get::<ProvidesFood>(e).unwrap());
            }
            if self.ecs_world.get::<ProvidesQuench>(e).is_ok(){
                saved.quench = Some(*self.ecs_world.get::<ProvidesQuench>(e).unwrap());
            }
            if self.ecs_world.get::<Equippable>(e).is_ok() {
                saved.equippable = Some(*self.ecs_world.get::<Equippable>(e).unwrap());
            }
            if self.ecs_world.get::<MeleeBonus>(e).is_ok(){
                saved.meleebonus = Some(*self.ecs_world.get::<MeleeBonus>(e).unwrap());
            }
            if self.ecs_world.get::<Equipped>(e).is_ok() {
                saved.equip = Some(*self.ecs_world.get::<Equipped>(e).unwrap()); 
            }

            save_datas.push(saved);
        }
        //'r' stands for Result
        let json_r = serde_json::to_string(&save_datas);
        log!("JSON: {:?} ", json_r);


        //map data
        let json_r2 = serde_json::to_string(&self.map);
        log!("JSON 2: {:?}", json_r2);

        //log!("{}", &format!("{}", serde_json::to_string(&self.player_position).unwrap()));
        // extract String from Result
        if json_r.is_ok() && json_r2.is_ok() {
            //hack because we can't return a tuple or Vec<> of Strings
            return json_r.unwrap() + " \nmap:" + &json_r2.unwrap();
        } else {
            return "".to_string();
        }
    }

    pub fn load_save(&mut self, data: String) {
        log!("Rust received loaded data {}", data);
        // split the string
        let split : Vec<&str> = data.split(" \nmap:").collect();
        // for s in split{
        //     log!("{}", &format!("Split {}", s));
        // }

        let res =  serde_json::from_str(&split[0]);
        if res.is_ok() {
            let ent: Vec<SaveData> = res.unwrap();
            for e in ent {
                //log!("Ent from save: {:?}", e);
                //log!("{}", &format!("Ent from save: {} {} {:?} {:?} {:?} {:?} {:?}", e.entity, e.name, e.render, e.point, e.item, e.backpack, e.equip));
                
                //entity handle
                let ent = hecs::Entity::from_bits(e.entity); //restore

                //build our entity from pieces listed
                let mut builder = hecs::EntityBuilder::new();
                builder.add(e.name);
                if e.render.is_some(){
                    builder.add(e.render.unwrap());
                }
                if e.point.is_some(){
                    builder.add(e.point.unwrap());
                }
                if e.player.is_some(){
                    builder.add(e.player.unwrap());
                    let point = e.point.unwrap();
                    self.player_position = self.map.xy_idx(point.x, point.y);
                }
                if e.needs.is_some(){
                    builder.add(e.needs.unwrap());
                }
                if e.ai.is_some(){
                    builder.add(e.ai.unwrap());
                }
                if e.money.is_some() {
                    builder.add(e.money.unwrap());
                }
                if e.faction.is_some() {
                    builder.add(e.faction.unwrap());
                }
                if e.vendor.is_some() {
                    builder.add(e.vendor.unwrap());
                }
                if e.combat.is_some(){
                    builder.add(e.combat.unwrap());
                }
                if e.item.is_some(){
                    builder.add(e.item.unwrap());
                }
                if e.backpack.is_some(){
                    builder.add(e.backpack.unwrap());
                }
                if e.consumable.is_some(){
                    builder.add(e.consumable.unwrap());
                }
                if e.heals.is_some(){
                    builder.add(e.heals.unwrap());
                }
                if e.food.is_some(){
                    builder.add(e.food.unwrap());
                }
                if e.quench.is_some(){
                    builder.add(e.quench.unwrap());
                }
                if e.equippable.is_some(){
                    builder.add(e.equippable.unwrap());
                }
                if e.meleebonus.is_some(){
                    builder.add(e.meleebonus.unwrap());
                }
                if e.equip.is_some(){
                    builder.add(e.equip.unwrap());
                }

                // spawn based on loaded data
                // automatically despawns any existing entities with the ids
                self.ecs_world.spawn_at(ent, builder.build());
            }

            let current_position = self.map.idx_xy(self.player_position);
            // refresh FOV
            self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
            self.fov.compute_fov(&mut self.fov_data, current_position.0 as usize, current_position.1 as usize, 6, true);
        }
        //let ent: Vec<SaveData> = Vec:new();

        let res =  serde_json::from_str(&split[1]);
        if res.is_ok() {
            let mapa = res.unwrap();
            self.map = mapa;
        }
    }

}

//Methods not exposed to JS
impl Universe {
    pub fn spawn_entities(&mut self, list_spawns:Vec<(usize, String)>) {
        for entity in list_spawns.iter() {
            let pos = self.map.idx_xy(entity.0);
            self.spawn(pos.0, pos.1, entity.1.clone());
        }
    }


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

    pub fn entities_at(&self, x: usize, y: usize) -> Option<Entity> {
        let mut ent: Option<Entity> = None;
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &u8)>()
        .without::<InBackpack>().without::<Equipped>() //no ref/pointer here!
        .iter() {
            if point.x as usize == x && point.y as usize == y {
                ent = Some(id);
                break;
            }
        }
        return ent;
    }

    pub fn items_at(&self, x: usize, y: usize) -> Option<Entity> {
        let mut item: Option<Entity> = None;
        for (id, (point, it)) in self.ecs_world.query::<(&Point, &Item)>()
        .without::<InBackpack>().without::<Equipped>() //no ref/pointer here!!!
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
            //log!("{}", &format!("Item in inventory: {}", self.ecs_world.get::<&str>(id).unwrap().to_string()));
            //log!("{}", &format!("ID: {:?}", id));
            ids.push(id.to_bits()); //we can't get from id later on, yet
        }
        return ids;
    }

    pub fn pickup_item(&mut self, item: &Entity) {
        self.ecs_world.insert_one(*item, InBackpack{});
        self.items_in_inventory();
    }

    pub fn use_item(&mut self, user: &Entity, it: &Entity) {
        // The indirection is here to make it possible for non-player Entities to use items
        //tell the engine that we want to use the item
        self.ecs_world.insert_one(*user, WantsToUseItem{item:*it});

        //message
        game_message(&format!("{} used {}", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<String>(*it).unwrap().to_string()));
        // apply the use effects
        let mut wants : Vec<Entity> = Vec::new();
        let mut to_unequip : Vec<Entity> = Vec::new();
        for (id, (wantstouse)) in self.ecs_world.query::<(&WantsToUseItem)>().iter(){
            //log!("{}", &format!("Want to use item: {:?}", wantstouse.item));
            //log!("{}", &format!("Item: {}", self.ecs_world.get::<String>(wantstouse.item).unwrap().to_string()));

            // If it heals, apply the healing
            // NOTE: no & here!!!
            if self.ecs_world.get::<ProvidesHealing>(wantstouse.item).is_ok() {
                game_message(&format!("{{g{} heals {} damage", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<ProvidesHealing>(wantstouse.item).unwrap().heal_amount));                
            } else {
                log!("Item doesn't provide healing");
            }

            // food or drink?
            if self.ecs_world.get::<ProvidesQuench>(wantstouse.item).is_ok(){
                game_message(&format!("{{gYou drink the {}", self.ecs_world.get::<String>(*it).unwrap().to_string()));
            } else if self.ecs_world.get::<ProvidesFood>(wantstouse.item).is_ok(){
                game_message(&format!("{{gYou eat the {}", self.ecs_world.get::<String>(*it).unwrap().to_string()));
            }

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            if self.ecs_world.get::<Equippable>(wantstouse.item).is_ok() {
                let can_equip = self.ecs_world.get::<Equippable>(wantstouse.item).unwrap();
                let target_slot = can_equip.slot;
        
                // Remove any items the target has in the item's slot
                //let mut to_unequip : Vec<Entity> = Vec::new();

                //find items in slot
                for (ent_id, (equipped)) in self.ecs_world.query::<(&Equipped)>()
                .with::<String>()
                .iter()
                {
                    let owner = hecs::Entity::from_bits(equipped.owner);
                    if owner == *user && equipped.slot == target_slot {
                        to_unequip.push(ent_id);
                        //if target == *player_entity {
                        game_message(&format!("{{rYou unequip {}.", self.ecs_world.get::<String>(ent_id).unwrap().to_string()));
                    }   
                }
                wants.push(wantstouse.item);
                game_message(&format!("{{g{} equips {}", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<String>(*it).unwrap().to_string()));
            }

            if self.ecs_world.get::<Consumable>(wantstouse.item).is_ok() {
                log!("Item is a consumable");
                //FIXME: we can't add components or remove entities while iterating, so this is a hack
                self.ecs_world.get_mut::<ToRemove>(wantstouse.item).unwrap().yes = true;
            }
        }

        // deferred some actions because we can't add or remove components when iterating
        for item in to_unequip.iter() {
            self.ecs_world.remove_one::<Equipped>(*item);
        }

        for item in wants.iter() {
            let eq = { //scope to get around borrow checker
                let get = self.ecs_world.get::<Equippable>(*item).unwrap();
                *get //clone here to get around borrow checker
            };
            // slot related to item's slot
            self.ecs_world.insert_one(*item, Equipped{owner:user.to_bits(), slot:eq.slot});
            
            //self.ecs_world.remove_one::<InBackpack>(*item);
        }

    }


    //a very simple test, akin to flipping a coin or throwing a d2
    fn make_test_d2(&self, skill: u32) -> Vec<bool> {
        let mut rolls = Vec::new();
        for _ in 0..10-skill { // exclusive of end
            rolls.push(rand::random()) // generates a boolean
        }
        return rolls
    }

    fn attack(&self, target: &Entity) {
        let res = self.make_test_d2(1);
        let sum = res.iter().filter(|&&b| b).count(); //iter returns references and filter works with references too - double indirection
        game_message(&format!("Test: {} sum: {{g{}", Rolls(res), sum));

        if sum >= 5 {
            game_message(&format!("Attack hits!"));
            //item bonuses
            let mut offensive_bonus = 0;
            for (id, (power_bonus, equipped_by)) in self.ecs_world.query::<(&MeleeBonus, &Equipped)>().iter() {
                //if equipped_by.owner == attacker {
                    offensive_bonus += power_bonus.bonus;
            }

            //deal damage
            // the mut here is obligatory!!!
            let mut stats = self.ecs_world.get_mut::<CombatStats>(*target).unwrap();
            stats.hp = stats.hp - 2 - offensive_bonus;
            game_message(&format!("Dealt {{r{}}} damage", 2+offensive_bonus));
            
            //borrow checker doesn't allow this??
            //if killed, despawn
            // if stats.hp <= 0 {
            //     self.ecs_world.despawn(*target).unwrap();
            //     log!("{}", &format!("Target was killed!"));
            // }
        } else {
            game_message(&format!("Attack missed!"));
        }
    }

    fn is_player_dead(&self) -> bool {
        //check for dead
        let mut dead = false;
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => { 
                let hp = self.ecs_world.get::<CombatStats>(entity).unwrap().hp;
                if hp <= 0 {
                    dead = true;
                }
            },
            None => { dead = true },
        }
        return dead;
    }

    fn survival_tick(&mut self) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let mut needs = self.ecs_world.get_mut::<Needs>(entity).unwrap();
                needs.hunger -= 1;
                needs.thirst -= 1;
            },
            None => {},
        }
    }


    fn remove_dead(&mut self) {
        // Here we query entities with 0 or less hp and despawn them
        let mut to_remove: Vec<Entity> = Vec::new();
        let mut to_drop : Vec<(Entity, Point)> = Vec::new();
        for (id, stats) in &mut self.ecs_world.query::<&CombatStats>() {
            if stats.hp <= 0 {
                if id.id() > 0 { 
                    to_remove.push(id);
                }
                // player - just a log message
                else {
                    game_message("{rYou are DEAD!");
                }
            }
        }

        for (id, remove) in &mut self.ecs_world.query::<&ToRemove>() {
            if remove.yes {
                to_remove.push(id);
            }
        }

        for entity in to_remove {
            // not item
            if self.ecs_world.get::<Item>(entity).is_err() {
                //drop their stuff
                let pos = self.ecs_world.get::<Point>(entity).unwrap();
                for (ent_id, (equipped)) in self.ecs_world.query::<(&Equipped)>()
                .with::<String>()
                .iter()
                {
                    let owner = hecs::Entity::from_bits(equipped.owner);
                    if owner == entity {
                        to_drop.push((ent_id, *pos));
                    }
                }

                game_message(&format!("{{grAI {} is dead", self.ecs_world.get::<String>(entity).unwrap().to_string()));
            }
            
            self.ecs_world.despawn(entity).unwrap();
        }

        // deferred some actions because we can't add or remove components when iterating
        for it in to_drop.iter() {
            self.ecs_world.remove_one::<Equipped>(it.0);
            let mut pt = self.ecs_world.get_mut::<Point>(it.0).unwrap();
            pt.x = it.1.x;
            pt.y = it.1.y;
            //log!("{}", &format!("Dropping item {} x {} y {} ", self.ecs_world.get::<String>(it.0).unwrap().to_string(), pt.x, pt.y));
        }
    }
    

    
    pub fn get_AI(&mut self) {
        // we need to borrow mutably (for the movement to happen), so we have to use a Point instead of two usizes (hecs limitation)
        for (id, (ai, point)) in &mut self.ecs_world.query::<(&AI, &mut Point)>()
        .with::<String>()
        .iter()
         {
            log!("{}", &format!("Got AI {} x {} y {}",  point.x, point.y, self.ecs_world.get::<String>(id).unwrap().to_string())); //just unwrapping isn't enough to format
            //if the player's immediately next to us, don't run costly A*
            let player_pos = self.map.idx_xy(self.player_position);
            //log!("{}", &format!("Player pos x {} y {}", player_pos.0, player_pos.1));
            if distance2d_chessboard(point.x, player_pos.0, point.y, player_pos.1) < 2 {
                //log!("{}", &format!("AI next to player, attack!"));
                game_message(&format!("{{rAI {} kicked at the player", self.ecs_world.get::<String>(id).unwrap().to_string()));
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
                //can we see the player? (assumes symmetric FOV)
                if self.is_visible(point.x as usize, point.y as usize) {
                    let new_pos = path_to_player(&mut self.map, point.x as usize, point.y as usize, self.player_position);
                    // move or attack            
                    if new_pos.0 == player_pos.0 as usize && new_pos.1 == player_pos.1 as usize {
                        game_message(&format!("{{rAI {} kicked at the player", self.ecs_world.get::<String>(id).unwrap().to_string()));
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
