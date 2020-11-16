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

//save/load
use serde::{Serialize, Deserialize};
use serde_json::json;

//our stuff

//3rd party vendored in
mod fastnoise;
use fastnoise::*;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Renderable {
    Thug = 0,
    Knife = 1,
    Medkit = 2,
}

//for ECS
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Player{}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct AI {}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Item{}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InBackpack{}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Consumable{}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

//don't need to be serialized
pub struct WantsToUseItem {
    pub item : Entity
}
// tells the engine to nuke us
pub struct ToRemove {pub yes: bool} //bool is temporary while we can't modify entities when iterating

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee }
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

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    entity: u64, //because Entity cannot be serialized by serde
    name: String,
    point: Option<Point>,
    render: Option<u8>,
    player: Option<Player>,
    ai: Option<AI>,
    combat: Option<CombatStats>,
    item: Option<Item>,
    backpack: Option<InBackpack>,
    consumable: Option<Consumable>,
    heals: Option<ProvidesHealing>,
    equippable: Option<Equippable>,
    meleebonus: Option<MeleeBonus>,
    equip: Option<Equipped>,
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
    SaveGame,
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
    
        //mapgen

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
                log!("{}", &format!("Noise: x{}y{} {}", x, y, n));
                if n > 125.0 || n < -125.0 {
                    state.tiles[xy_idx(x,y)] = Cell::Wall as u8;
                    // opaque
                    state.fov_data.set_transparent(x as usize, y as usize, false);
                } else {
                    //state.tiles[xy_idx(x,y)] = Cell:Floor as u8
                }
                //log!("{}", &format!("Tile: x{} y{} {}", x,y, state.tiles[xy_idx(x,y)]));
            }
        }
      


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
        //reveal tiles
        for (idx, b) in state.fov_data.fov.iter().enumerate() {
            if *b {
                state.map.revealed_tiles[idx] = true;
            }
        }
        
        //rendering and position handled otherwise, so the player Entity only needs combat stats
        let player = state.ecs_world.spawn(("Player".to_string(), Player{}, CombatStats{hp:20, max_hp: 20, defense:1, power:1}));

        //spawn entities
        let a = state.ecs_world.spawn((Point{x:4, y:4}, Renderable::Thug as u8, "Thug".to_string(), AI{}, CombatStats{hp:10, max_hp:10, defense:1, power:1}));
        let it = state.ecs_world.spawn((Point{x:6,y:7}, Renderable::Knife as u8, "Combat knife".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Melee }, MeleeBonus{ bonus: 2}, ToRemove{yes:false}));
        let med = state.ecs_world.spawn((Point{x:5, y:5}, Renderable::Medkit as u8, "Medkit".to_string(), Item{}, ToRemove{yes:false}, Consumable{}, ProvidesHealing{heal_amount:5}));

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

    pub fn is_seen(&self, x: usize, y:usize) -> bool {
        return self.map.revealed_tiles[xy_idx(x as i32, y as i32)];
    }

    pub fn should_draw(&self, x: usize, y:usize) -> bool {
        return self.is_visible(x,y) || self.is_seen(x,y);
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

                    //save/load
                    //Command::SaveGame => self.save_game(),

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
                    game_message(&format!("Player kicked the {}", self.ecs_world.get::<String>(entity).unwrap().to_string()));
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
                    //reveal tiles
                    for (idx, b) in self.fov_data.fov.iter().enumerate() {
                        if *b {
                            self.map.revealed_tiles[idx] = true;
                        }
                    }
                    //enemy turn
                    self.get_AI();
                    self.remove_dead();
                }
            }
                 
        }
        else {
            log!("{}", &format!("Blocked move to {}, {} ", new_position.0,new_position.1))
        }
    }

    pub fn get_item(&mut self) {
        let current_position = idx_xy(self.player_position);
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
                log!("Player uses item {}", id);
                let item = hecs::Entity::from_bits(id); //restore
                self.use_item(&entity, &item);
                self.remove_dead(); //in case we used a consumable item
            },
            None => {},
        }
    }

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
                name: self.ecs_world.get::<String>(e).unwrap().to_string(),
                player: None,
                ai: None,
                combat: None,
                item: None,
                backpack: None,
                consumable: None,
                heals: None,
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

            //those aren't guaranteed
            if self.ecs_world.get::<Player>(e).is_ok() {
                //log!("{:?} is player", e);
                saved.player = Some(*self.ecs_world.get::<Player>(e).unwrap());
                //save player position
                let current_position = idx_xy(self.player_position);
                saved.point = Some(Point{x:current_position.0, y:current_position.1});
            }
            if self.ecs_world.get::<AI>(e).is_ok(){
                saved.ai = Some(*self.ecs_world.get::<AI>(e).unwrap());
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
        log!("{}", &format!("{}", serde_json::to_string(&self.player_position).unwrap()));
        // extract String from Result
        if json_r.is_ok() {
            return json_r.unwrap();
        } else {
            return "".to_string();
        }
    }

    pub fn load_save(&mut self, data: String) {
        log!("Rust received loaded data {}", data);
        let res =  serde_json::from_str(&data);
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
                    self.player_position = xy_idx(point.x, point.y);
                }
                if e.ai.is_some(){
                    builder.add(e.ai.unwrap());
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

            let current_position = idx_xy(self.player_position);
            // refresh FOV
            self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
            self.fov.compute_fov(&mut self.fov_data, current_position.0 as usize, current_position.1 as usize, 6, true);
        }
        //let ent: Vec<SaveData> = Vec:new();
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
                game_message(&format!("{} heals {} damage", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<ProvidesHealing>(wantstouse.item).unwrap().heal_amount));                
            } else {
                log!("Item doesn't provide healing");
            }
            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            if self.ecs_world.get::<Equippable>(wantstouse.item).is_ok() {
                let can_equip = self.ecs_world.get::<Equippable>(wantstouse.item).unwrap();
                let target_slot = can_equip.slot;
        
                // Remove any items the target has in the item's slot
                //let mut to_unequip : Vec<Entity> = Vec::new();

                //find items in slot
                for (ent_id, (equipped)) in self.ecs_world.query::<(&Equipped)>()
                .with::<&str>() //we can't query it directly above because str length is unknown at compile time
                .iter()
                {
                    let owner = hecs::Entity::from_bits(equipped.owner);
                    if owner == *user && equipped.slot == target_slot {
                        to_unequip.push(ent_id);
                        //if target == *player_entity {
                        game_message(&format!("You unequip {}.", self.ecs_world.get::<String>(ent_id).unwrap().to_string()));
                    }   
                }
                wants.push(wantstouse.item);
                game_message(&format!("{} equips {}", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<String>(*it).unwrap().to_string()));
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
        for _ in 0..20-skill { // exclusive of end
            rolls.push(rand::random()) // generates a boolean
        }
        return rolls
    }

    fn attack(&self, target: &Entity) {
        let res = self.make_test_d2(1);
        let sum = res.iter().filter(|&&b| b).count(); //iter returns references and filter works with references too - double indirection
        log!("{}", &format!("Test: {:?} sum: {}", res, sum));

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
        game_message(&format!("Dealt {} damage", 2+offensive_bonus));
        
        //borrow checker doesn't allow this??
        //if killed, despawn
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

        for (id, remove) in &mut self.ecs_world.query::<&ToRemove>() {
            if remove.yes {
                to_remove.push(id);
            }
        }

        for entity in to_remove {
            if self.ecs_world.get::<Item>(entity).is_err() {
                game_message(&format!("AI {} is dead", self.ecs_world.get::<String>(entity).unwrap().to_string()));
            }
            
            self.ecs_world.despawn(entity).unwrap();
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
            let player_pos = idx_xy(self.player_position);
            //log!("{}", &format!("Player pos x {} y {}", player_pos.0, player_pos.1));
            if distance2d_chessboard(point.x, player_pos.0, point.y, player_pos.1) < 2 {
                //log!("{}", &format!("AI next to player, attack!"));
                game_message(&format!("AI {} kicked at the player", self.ecs_world.get::<String>(id).unwrap().to_string()));
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
