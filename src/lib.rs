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
//use rand::Rng;

//save/load
use serde::{Serialize, Deserialize};

use std::fmt;

//time
use chrono::{NaiveTime, Timelike, Duration};

//our stuff

//3rd party vendored in
mod fastnoise;

mod universe_private;
use universe_private::*;

mod map;
use map::*;
mod fov;
use fov::*;
mod astar;
//use astar::*;
mod utils;
use utils::*;
mod rect;
use rect::*;
mod map_builders;
//use map_builders::*;

mod ai;

mod saveload;
mod data_loader;

//lisp-y
mod lispy;
use lispy::*;

#[macro_use]
extern crate lazy_static;

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

    let log_history = document.get_element_by_id("log-history").unwrap();

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

    //clone to place a copy
    let ln = line.clone_node().unwrap().dyn_into::<web_sys::HtmlElement>().unwrap(); //Rust version for some reason doesn't have the deep parameter?
    ln.set_inner_html(&string); //because clone_node doesn't do it for some reason
    log_history.append_child(&ln).unwrap();

    //axe the first if more than 5
    while messages.child_element_count() > 5 {
        messages.remove_child(&messages.first_element_child().unwrap()); //implicit conversion
    }
}	

pub fn game_describe(string: &str) {
    //convert
    //let mut string = string.to_string();
    let window = web_sys::window().expect("global window does not exists");    
    let document = window.document().expect("expecting a document on window");

    let desc = document.get_element_by_id("game-desc").unwrap();
    //let line = document.create_element("div").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap(); //dyn_into for style() to work

    desc.set_inner_html(string);
}


#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderableGlyph {
    Thug = 0,
    Knife = 1,
    Medkit = 2,
    Barkeep = 3,
    Table = 4,
    Chair = 5,
    Boots = 6,
    Jacket = 7,
    Jeans = 8,
    Patron = 9,
    Bed = 10
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderOrder {
    Actor = 1,
    Item = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Renderable {
    glyph: u8,
    order: RenderOrder,
}


//for ECS
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameState{
    pub turns: i64, //to fit chrono
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Player{}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Needs{
    pub hunger: i32,
    pub thirst: i32,
}
pub struct Path{
    pub steps: Vec<usize> // see astar line 43
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AI {
    //pub state: i32;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Faction {
    pub typ: FactionType
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vendor {
    //pub categories : Vec<String>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Asleep {}

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
pub struct WantsToDropItem {
    pub item : Entity
}
// tells the engine to nuke us
pub struct ToRemove {pub yes: bool} //bool is temporary while we can't modify entities when iterating

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee, Torso, Legs, Feet }
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DefenseBonus {
    pub bonus : f32
}

pub struct Conversation {
    pub text: String,
    pub answers: Vec<String>
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

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub enum WaitType {
    Minutes5,
    Minutes30,
    Hour1,
    Hour2,
    TillDusk,
}


//input
//for input that does not come from JS side (e.g. actions after conversation)
pub static mut GLOBAL_INPUT: Option<Command> = None;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Command {
    None, //dummy, unfortunately we can't use -1 in wasm_bindgen...
    MoveLeft,
    MoveRight,
    MoveDown,
    MoveUp,
    GetItem,
    Inventory,
    SaveGame,
    Wait,
    Rest,
}


#[wasm_bindgen]
pub struct Universe {
    map: Map,
    player_position: usize,
    fov: FovRecursiveShadowCasting,
    fov_data: MapData,
    ecs_world: World,
}
//can't store rng here because of wasm_bindgen


/// Public methods, exported to JavaScript.
#[wasm_bindgen]
// returning Universe as a workaround for https://github.com/rustwasm/wasm-bindgen/issues/1858
pub async fn load_datafile_ex(mut state: Universe) -> Universe {
    return data_loader::load_datafile(state).await;
}


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

        //state.spawn_entities();

        log!("We have a universe");

        //lispy test
        //parse_script();

        // We'll return the state with the short-hand
        state
    }


    //for JS follow-ups (the main function isn't exposed)
    pub fn on_game_start(&mut self) {
        //show MUD desc for initial position
        let current_position = self.map.idx_xy(self.player_position);
        self.text_description(self.player_position, current_position.0, current_position.1);
        //greet the player
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

    pub fn get_pos(&self, ind: usize) -> Vec<i32> {
        let pos = self.map.idx_xy(ind);
        vec![pos.0, pos.1]
    }

    pub fn draw_entities(&self) -> Vec<u8> {
        // Each "drawn" will store 3 u8 values (x,y and tile)
        // based on https://aimlesslygoingforward.com/blog/2017/12/25/dose-response-ported-to-webassembly/ 
        let mut js_drawn = Vec::new();

        let mut data = self.ecs_world.query::<(&Point, &Renderable)>()
        .without::<InBackpack>().without::<Equipped>() //no ref/pointer here!
        .iter()
        .map(|(e, (&p, &r))| (e, p, r)) // Copy out of the world
        .collect::<Vec<_>>();

        //sort by render order
        data.sort_by(|&a, &b| (b.2.order as u8).cmp(&(a.2.order as u8)) );
        //data.sort_by(|&a, &b| (a.2.order as u8).cmp(&(b.2.order as u8)) );
        //log!("{}", format!("{:?}", data));

        for (id, point, render) in data.iter() {
            if self.is_visible(point.x as usize, point.y as usize) {
                js_drawn.push(point.x as u8);
                js_drawn.push(point.y as u8);
                js_drawn.push(render.glyph);
                //log!("{}", &format!("Rust: x {} y {} tile {}", point.x, point.y, render.glyph));
            }
        }

        // for (id, (point, render)) in self.ecs_world.query::<(&Point, &u8)>()
        // .without::<InBackpack>().without::<Equipped>() //no ref/pointer here!
        // .iter() {
        //     if self.is_visible(point.x as usize, point.y as usize) {
        //         js_drawn.push(point.x as u8);
        //         js_drawn.push(point.y as u8);
        //         js_drawn.push(*render);
        //         //log!("{}", &format!("Rust: x {} y {} tile {}", point.x, point.y, render));
        //     }
        // }

        return js_drawn;
    }

    //for JS (currently unused because wasm_bindgen doesn't play nice with Vec<NPCPrefab>)
    // pub fn spawn_ex(&mut self, x:i32, y:i32, name:String) {
    //     let pos = self.map.free_grid_in_range(x,y,4);
    //     return self.spawn(pos.x,pos.y,name);
    // }

    pub fn console_input(&mut self, input:String) {
        log!("Rust console input: {}", input);
        
        self.debug_console_core(input);
    }


    pub fn process(&mut self, mut input: Option<Command>) {
        //new: handle other sorts of input sources
        //UGLY but I don't know a better way to do it, it's based on bracketlib input handling
        unsafe {
            if GLOBAL_INPUT.is_some() {
                input = GLOBAL_INPUT;
                log!("Global input: {:?}", input);
            }

        }
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
                    Command::Rest => self.rest(),
                    
                    //Command::Wait => self.wait(),
                    //save/load
                    //Command::SaveGame => self.save_game(),

                    _ => {} // Ignore all the other possibilities
                }
            }
        }

        //clear
        unsafe {
            GLOBAL_INPUT = None;
        }

    }

    pub fn astar_path(&mut self, x:i32, y:i32) {
        if self.is_player_dead() {
            return;
        }
        let mut new_path = player_path_to_target(&mut self.map,  self.player_position, x as usize, y as usize);

        //bugfix
        if !new_path.contains(&self.player_position){
            new_path.insert(0, self.player_position);
        }

        //debugging
        for i in &new_path {
            log!("{}", &format!("x {} y {}", self.map.idx_xy(*i).0, self.map.idx_xy(*i).1));
        }

        self.set_automove(new_path);

    }

    // Handle player movement. Delta X and Y are the relative move
    // requested by the player. We calculate the new coordinates,
    // and if it is a floor - move the player there.
    pub fn move_player(&mut self, delta_x: i32, delta_y: i32) {
        //log!("Move player x {} y {}", delta_x, delta_y);

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
                        } 
                        else if self.ecs_world.get::<Conversation>(entity).is_ok() {
                            let conv = self.ecs_world.get::<Conversation>(entity).unwrap();
                            //display convo
                            let window = web_sys::window().expect("global window does not exists");    
                            let document = window.document().expect("expecting a document on window");                        
                            let view = document.get_element_by_id("conversation").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
                            
                            //the img is there to mark language being spoken by NPC
                            let text = &format!("<div> <img src=\"./es.svg\" alt=\"\" style=\"height:14px\"> {} </div>", conv.text);
                            let mut replies = "".to_string();
                            for (i, a) in conv.answers.iter().enumerate() {
                                let tmp = format!("<button id=conv-id-{}>{}</button> <span style=\"color:rgb(0,255,0)\"> {} <span>", i, i, a);
                                replies = format!("{} \n {}", replies, tmp);
                            }  
                            // ... and player
                            view.set_inner_html(&format!("{} \n <img src=\"./es.svg\" alt=\"\" style=\"height:14px\"> {}", text, replies));


                            //basic interactivity
                            for (i,a) in conv.answers.iter().enumerate() {
                                //closure
                                //needs move due to i being used
                                let click_handle =  Closure::wrap(Box::new(move || {
                                    //log!("Test click handler");
                                    log!("Clicked button for answer id {}", i);
                                    
                                    //close the menu for now
                                    //get the damned thing by ourselves to avoid 'value moved'
                                    let window = web_sys::window().expect("global window does not exists");    
                                    let document = window.document().expect("expecting a document on window");  
                                    let view = document.get_element_by_id("conversation").unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
                                    view.class_list().toggle("visible");

                                    //test
                                    // unsafe {
                                    //     GLOBAL_INPUT = Some(Command::MoveLeft);
                                    //     log!("Command to move left issued");
                                    // }
                                    
                                }) as Box<dyn FnMut()>);

                                let id = &format!("conv-id-{}", i);
                                let but = document.get_element_by_id(id).unwrap().dyn_into::<web_sys::HtmlElement>().unwrap();
                                but.set_onclick(Some(click_handle.as_ref().unchecked_ref()));

                                //avoid memleak on Rust side
                                click_handle.forget();
                            }

                            let list = view.class_list().toggle("visible");
                           
                        }
                        else {
                            game_message(&format!("The man says 🇪 🇸: hola!"));
                        }
                    }

                    //enemy turn
                    self.end_turn();
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

                    self.text_description(new_idx, new_position.0, new_position.1);

                    //enemy turn
                    self.end_turn();
                }
            }
                 
        }
        else {
            log!("{}", &format!("Blocked move to {}, {} ", new_position.0,new_position.1))
        }
    }

    pub fn set_automove(&mut self, path: Vec<usize>) {
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

    pub fn get_automove(&self) -> Vec<usize> {
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
                    
                    //paranoia check
                    if !steps.contains(&self.player_position) {  
                        // for i in &steps {
                        //      log!("{}", &format!("x {} y {}", self.map.idx_xy(*i as usize).0, self.map.idx_xy(*i as usize).1));
                        // }
                    //if steps[0] as usize != self.player_position {
                        log!("{}", &format!("Player pos x {} y {} not in steps", self.map.idx_xy(self.player_position).0, self.map.idx_xy(self.player_position).1));
                        return [].to_vec();
                    }
                    else {
                        steps.remove(0);
                        return steps;
                    }
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

    //MUD-style description
    pub fn text_description(&self, new_idx: usize, new_x: i32, new_y: i32){
        let new_position = (new_x, new_y);


        let area_desc = "This area appears to be a town that hugs a forest.";
        let mut terrain_desc = "";
        if (self.map.tiles[new_idx] == Cell::Grass as u8){
            terrain_desc = "You feel the grass under your feet.";
        }
        else if (self.map.tiles[new_idx] == Cell::Floor as u8) {
            terrain_desc = " You walk on paved ground of the town.";
        }
        else if (self.map.tiles[new_idx] == Cell::FloorIndoor as u8) {
            terrain_desc = " You entered one of the buildings.\n";
        }

        //describe the doors/walls in sight
        let mut other_desc = "".to_string();
        let mut has_walls = false;
        for (idx, b) in self.map.revealed_tiles.iter().enumerate() {
            if *b {
                //log!("Idx {} map {} ", idx, self.map.tiles[idx]);
                if (self.map.tiles[idx] == Cell::Door as u8) {
                    let point = Point{x:self.map.idx_xy(idx).0, y:self.map.idx_xy(idx).1};
                    // the range of the viewport to each side is x 20 y 12
                    if (point.x-new_position.0).abs() <= 20 && (point.y-new_position.1).abs() <= 12 {
                        let dist = distance2d_chessboard(point.x, point.y, new_position.0, new_position.1);
                        let direction = dir(&Point{x:new_position.0, y:new_position.1}, &Point{x:point.x, y:point.y});
                        // door is not necessarily the first thing you see, so we need to keep any existing other_desc
                        let tmp = format!(" You see a door {} away to {:?}.", dist, direction); 
                        other_desc = format!("{} {}", other_desc, tmp)
                    }

                }
                if (self.map.tiles[idx] == Cell::Wall as u8) {
                    let point = Point{x:self.map.idx_xy(idx).0, y:self.map.idx_xy(idx).1};
                    // the range of the viewport to each side is x 20 y 12
                    if (point.x-new_position.0).abs() <= 20 && (point.y-new_position.1).abs() <= 12 {
                        let dist = distance2d_chessboard(point.x, point.y, new_position.0, new_position.1);
                        let direction = dir(&Point{x:new_position.0, y:new_position.1}, &Point{x:point.x, y:point.y});
                        let mut tmp = format!(" and {} away to {:?},", dist, direction);
                        if !has_walls {
                            tmp = format!(" You see a wall {} away to {:?},", dist, direction);
                        }
                        other_desc = format!("{} {}", other_desc, tmp);
                        has_walls = true;
                    }
                }
            }
        }

        //describe entities in view
        let mut ent_desc = "You see here:".to_string();
        for (id) in self.view_list().iter() {
            let tmp = self.view_string_for_id(*id);
            ent_desc = format!("{} {}", ent_desc, tmp);
        }

        game_describe(&format!("{} {} {}\n {}", area_desc, terrain_desc, other_desc, ent_desc));
        
    }

    //GUI stuff
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

    pub fn view_list(&self) -> Vec<u64> {
        let mut list = Vec::new();
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &Renderable)>()
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
        let dist = distance2d_chessboard(point.x, point.y, player_pos.0, player_pos.1);
        let name = self.ecs_world.get::<String>(ent).unwrap().to_string();
        return format!("{} - {} {:?}", name, dist, direction);
    }

    pub fn entity_view_pos(&self, id: u64) -> Vec<i32> {
        let ent = hecs::Entity::from_bits(id); //restore
        let point = self.ecs_world.get::<Point>(ent).unwrap();
        return vec![point.x, point.y];
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
                self.end_turn();
            },
            None => {},
        }
    }

    pub fn drop_item_ext(&mut self, id: u64) {
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

                log!("Player drops item {}", id);
                let item = hecs::Entity::from_bits(id); //restore
                self.drop_item(&entity, &item);
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
            item = Some(self.ecs_world.spawn((Point{x:current_position.0,y:current_position.1}, Renderable{glyph: RenderableGlyph::Medkit as u8, order: RenderOrder::Item}, "Protein shake".to_string(), Item{}, ProvidesFood{}, ProvidesQuench{}, Consumable{}, ToRemove{yes:false})));
        }
        if name == "Medkit".to_string() {
            item = Some(self.ecs_world.spawn((Point{x:5, y:5}, Renderable{glyph:RenderableGlyph::Medkit as u8, order:RenderOrder::Item}, "Medkit".to_string(), Item{}, ToRemove{yes:false}, Consumable{}, ProvidesHealing{heal_amount:5})));
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

    pub fn get_faction(&self, x:i32, y:i32) -> u8 {
        let mut fact = 99; //unfortunately we can't use -1 in u8
        let ent = self.entities_at(x as usize, y as usize);
        match ent {
            Some(entity) => {
                if self.ecs_world.get::<Faction>(entity).is_ok() {
                    fact = self.ecs_world.get::<Faction>(entity).unwrap().typ as u8;
                }
            },
            None => { }
        }
        
        return fact;
    }

    pub fn rest(&mut self) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let turns_passed = Duration::hours(8).num_seconds();

                //simulate all that time
                for _ in 0..turns_passed {
                    self.get_AI();
                    //reenable when it makes sense to do so
                    //no inter-AI combat yet
                    //self.remove_dead();
                    // needs rebalancing for 1s turns
                    //self.survival_tick();
                }

                let mut gs = self.ecs_world.get_mut::<GameState>(entity).unwrap();
                //add the current number of turns to game start
                let cur_t = NaiveTime::from_hms(08, 00, 00).overflowing_add_signed(Duration::seconds(gs.turns));
                let t = cur_t.0.overflowing_add_signed(Duration::hours(8));

                // //t is a tuple (NaiveTime, i64)
                let f = t.0.format("%H:%M:%S").to_string();
                game_message(&format!("Time: {}", f));

                //let mut gs = self.ecs_world.get_mut::<GameState>(entity).unwrap();
                //update our turns counter
                gs.turns += turns_passed;

            },
            None => {},
        }
    }    

    pub fn wait(&mut self, opt: WaitType) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let mut turns_passed = 0;

                if opt == WaitType::TillDusk {
                    // block to make the loop below work
                    {
                        let gs = self.ecs_world.get_mut::<GameState>(entity).unwrap();

                        //wait until 19:00
                        let end_t = NaiveTime::from_hms(19,00,00);
                        //let mut f = end_t.format("%H:%M:%S").to_string();
                        //game_message(&format!("End time: {}", f));

                        //add the current number of turns to game start
                        let cur_t = NaiveTime::from_hms(08, 00, 00).overflowing_add_signed(Duration::seconds(gs.turns));
                        //returns a Duration
                        let diff = end_t - cur_t.0;
                        turns_passed = diff.num_seconds();
                        //log!("{}", &format!("{} s", diff.num_seconds()));
                        //gs.turns += turns_passed;    
                    }
                }

                if opt == WaitType::Minutes5 {
                    turns_passed = Duration::minutes(5).num_seconds();
                }
                if opt == WaitType::Minutes30 {
                    turns_passed = Duration::minutes(30).num_seconds();
                }
                if opt == WaitType::Hour1 {
                    turns_passed = Duration::hours(1).num_seconds();
                }
                if opt == WaitType::Hour2 {
                    turns_passed = Duration::hours(2).num_seconds();
                }
               
                //simulate all that time
                for _ in 0..turns_passed {
                    self.get_AI();
                    //reenable when it makes sense to do so
                    //no inter-AI combat yet
                    //self.remove_dead();
                    // needs rebalancing for 1s turns
                    //self.survival_tick();
                }


                //calculate time again
                let mut gs = self.ecs_world.get_mut::<GameState>(entity).unwrap();
                gs.turns += turns_passed; // add it here because otherwise the AI acts on final time
                let cur_t = NaiveTime::from_hms(08, 00, 00).overflowing_add_signed(Duration::seconds(gs.turns));
                // //t is a tuple (NaiveTime, i64)
                let f = cur_t.0.format("%H:%M:%S").to_string();
                game_message(&format!("Time: {}", f));
            },
            None => {},
        }
    }

    pub fn save_game(&self) -> String {
        log!("Saving game...");
        return saveload::save_game(self);
    }

    pub fn load_save(&mut self, data: String) {
        saveload::load_save(self, data);
        // refresh FOV
        let current_position = self.map.idx_xy(self.player_position);
        self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
        self.fov.compute_fov(&mut self.fov_data, current_position.0 as usize, current_position.1 as usize, 6, true);
    }

}
///-------------------------------------------------------------------------------------------------------------



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
