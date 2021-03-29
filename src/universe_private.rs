use super::log;
use super::{game_message, 
    Universe, 
    Cell, Renderable, RenderableGlyph, RenderOrder, Rolls,
    ToRemove,
    Point, Player, GameState, Needs, Money, Path,
    AI, Vendor, CombatStats, Faction, FactionType, Conversation, Attributes, Attribute,
    WantsToDropItem, WantsToUseItem,
    Item, InBackpack, Consumable, ProvidesHealing, ProvidesFood, ProvidesQuench, Equippable, EquipmentSlot, MeleeBonus, DefenseBonus, Equipped};

//ECS
use hecs::World;
use hecs::Entity;

//RNG
use rand::Rng;

//time
use chrono::{NaiveTime, Timelike, Duration};

use super::data_loader::{DataMaster, NPCPrefab, DATA};
use super::map_builders;    
use super::map::*;
use super::fov::*;
use super::astar::a_star_search;
use super::utils::*;

//it's outside Universe because we're careful not to pass 'self' to it
pub fn path_to_player(map: &mut Map, x: usize, y: usize, player_position: usize) -> (usize, usize) {
    //call A*
    let path = a_star_search(map.xy_idx(x as i32, y as i32), player_position, &map);
    if path.success {
        let idx = path.steps[1];
        let idx_pos = map.idx_xy(idx);
        if !map.is_tile_blocked(idx) {
            let old_idx = (y * map.width as usize) + x;
            //mark as blocked for pathfinding
            map.clear_tile_blocked(old_idx);
            map.set_tile_blocked(idx);
            log!("{}", &format!("Path step x {} y {}", idx_pos.0, idx_pos.1));
            return (idx_pos.0 as usize, idx_pos.1 as usize);
        }
    }
    log!("{}", &format!("No path found sx {} sy {} tx {} ty {}", x, y, map.idx_xy(player_position).0, map.idx_xy(player_position).1));
    (x,y) //dummy
}

pub fn player_path_to_target(map: &mut Map, player_position: usize, x: usize, y: usize) -> Vec<usize> {
    let path = a_star_search(player_position, map.xy_idx(x as i32, y as i32), &map);
    if path.success {
        return path.steps;
    }
    log!("{}", &format!("No player path found, x {} y {}", x,y));
    vec![player_position] //dummy
}

pub fn path_to_target(map: &mut Map, sx: usize, sy: usize, tx: usize, ty: usize) -> Vec<usize> {
    //call A*
    let path = a_star_search(map.xy_idx(sx as i32, sy as i32), map.xy_idx(tx as i32, ty as i32), &map);
    if path.success {
        return path.steps;
    }
    log!("{}", &format!("No path found sx {} sy {} tx {} ty {}", sx, sy, tx, ty));
    vec![map.xy_idx(sx as i32,sy as i32)] //dummy
}

//Methods not exposed to JS
impl Universe {
    pub fn game_start(&mut self, data: &DataMaster) {
        //mapgen
        let mut builder = map_builders::random_builder(80,60);
        builder.build_map();
        self.map = builder.build_data.map.clone();

        //spawn player on start
        match builder.build_data.starting_position {
            None => {},
            Some(point) => {
                self.player_position = self.map.xy_idx(point.x, point.y);
            }
        }

        //FOV
        self.fov_data = MapData::new(80,60);

        //build FOV cache
        for (idx, tile) in self.map.tiles.iter().enumerate() {
            if *tile == Cell::Wall as u8 {
                self.fov_data.set_transparent(self.map.idx_xy(idx).0 as usize, self.map.idx_xy(idx).1 as usize, false);
            }
        }
    
        self.fov_data.clear_fov(); // compute_fov does not clear the existing fov
        self.fov.compute_fov(&mut self.fov_data, self.map.idx_xy(self.player_position).0 as usize, self.map.idx_xy(self.player_position).1 as usize, 6, true);
        //reveal tiles
        for (idx, b) in self.fov_data.fov.iter().enumerate() {
            if *b {
                self.map.revealed_tiles[idx] = true;
            }
        }
        
        //rendering and position handled otherwise, so the player Entity only needs combat stats
        //NOTE: player is always entity id 0
        // 15, 14, 13, 12, 10, 8 aka elite array
        let player = self.ecs_world.spawn(("Player".to_string(), Player{}, GameState{turns:0}, CombatStats{hp:20, max_hp: 20, defense:1, power:1}, Money{money:100.0}, Needs{hunger:500, thirst:300}, 
        Attributes{strength:Attribute{base:2, bonus:0}, dexterity:Attribute{base:1, bonus:0}, constitution:Attribute{base:2, bonus:0}, intelligence:Attribute{base:1,bonus:0}, wisdom:Attribute{base:-1,bonus:0}, charisma:Attribute{base:0,bonus:0}}));
        //starting inventory
        self.give_item("Protein shake".to_string());
        self.give_item("Medkit".to_string());

        //spawn anything listed
        self.spawn_entities_list(builder.build_data.list_spawns, &data.npcs);
        self.spawn_entities(&data.npcs);
    }

    //moved spawn because of //https://github.com/rustwasm/wasm-bindgen/issues/111 preventing using vec<NPCPrefab> as parameter, too :(

    //TODO: unhardcode order?
    pub fn spawn(&mut self, x:i32, y:i32, name:String, data: &Vec<NPCPrefab>) {
        //TODO: should be a dict lookup
        // props
        if name == "Table".to_string() {
            self.ecs_world.spawn((Point{x:x, y:y}, Renderable{glyph: RenderableGlyph::Table as u8, order: RenderOrder::Item}));
        } else if name == "Chair".to_string() {
            self.ecs_world.spawn((Point{x:x, y:y}, Renderable{glyph: RenderableGlyph::Chair as u8, order: RenderOrder::Item}));
        }
        else if name == "Bed".to_string() {
            self.ecs_world.spawn((Point{x:x, y:y}, Renderable{glyph: RenderableGlyph::Bed as u8, order: RenderOrder::Item}));
        }
        //NPCs
        else if name == "Barkeep".to_string() {
            self.ecs_world.spawn((Point{x:x, y:y}, Renderable{glyph:data[1].renderable as u8, order: RenderOrder::Actor}, data[1].name.to_string(), data[1].faction.unwrap(), data[1].combat.unwrap(), Vendor{}));
            //self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Barkeep as u8, "Barkeep".to_string(), Faction{typ: FactionType::Townsfolk}, CombatStats{hp:5, max_hp:5, defense:1, power:1}, Vendor{}));
        } 
        else if name == "Patron".to_string() {
            let pat = self.ecs_world.spawn((Point{x:x, y:y}, Renderable{glyph:data[2].renderable as u8, order: RenderOrder::Actor}, data[2].name.to_string(), data[2].ai.unwrap(), data[2].faction.unwrap(), data[2].combat.unwrap()));
            //let pat = self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Patron as u8, "Patron".to_string(), AI{}, Faction{typ: FactionType::Townsfolk}, CombatStats{hp:3, max_hp:3, defense:1, power:1}));
            let conv = self.ecs_world.insert_one(pat, Conversation{text:"Hola, tio!".to_string(), answers:vec!["Tambien.".to_string(), "No recuerdo español.".to_string()]});
        } else if name == "Thug".to_string() {
            let th = self.ecs_world.spawn((Point{x:x, y:y}, Renderable{glyph:data[0].renderable as u8, order: RenderOrder::Actor}, data[0].name.to_string(), data[0].ai.unwrap(), data[0].faction.unwrap(), data[0].combat.unwrap()));
            //let th = self.ecs_world.spawn((Point{x:x, y:y}, Renderable::Thug as u8, "Thug".to_string(), AI{}, Faction{typ: FactionType::Enemy}, CombatStats{hp:10, max_hp:10, defense:1, power:1}));
            //their starting equipment
            let boots = self.ecs_world.spawn((Point{x:x, y:y}, Renderable{glyph:RenderableGlyph::Boots as u8, order: RenderOrder::Item}, "Boots".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Feet }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
            let l_jacket = self.ecs_world.spawn((Point{x:x,y:y}, Renderable{glyph: RenderableGlyph::Jacket as u8, order: RenderOrder::Item}, "Leather jacket".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Torso }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
            let jeans = self.ecs_world.spawn((Point{x:x,y:y}, Renderable{glyph: RenderableGlyph::Jeans as u8, order: RenderOrder::Item}, "Jeans".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Legs}, DefenseBonus{ bonus:0.1}, ToRemove{yes:false}));
            self.ecs_world.insert_one(boots, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Feet});
            self.ecs_world.insert_one(l_jacket, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Torso});
            self.ecs_world.insert_one(jeans, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Legs});
        }
        else {
            log!("Tried to spawn {}", name);
        }
    }


    pub fn spawn_entities(&mut self, data: &Vec<NPCPrefab>) {
        //spawn entities
        let th = self.ecs_world.spawn((Point{x:5,y:5}, Renderable{glyph:data[0].renderable as u8, order: RenderOrder::Actor}, data[0].name.to_string(), data[0].ai.unwrap(), data[0].faction.unwrap(), data[0].combat.unwrap()));

        //let th = self.ecs_world.spawn((Point{x:4, y:4}, Renderable::Thug as u8, "Thug".to_string(), AI{}, Faction{typ: FactionType::Enemy}, CombatStats{hp:10, max_hp:10, defense:1, power:1}));
        //their starting equipment
        let boots = self.ecs_world.spawn((Point{x:4, y:4}, Renderable{glyph: RenderableGlyph::Boots as u8, order: RenderOrder::Item}, "Boots".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Feet }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let l_jacket = self.ecs_world.spawn((Point{x:4,y:4}, Renderable{glyph: RenderableGlyph::Jacket as u8, order: RenderOrder::Item}, "Leather jacket".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Torso }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let jeans = self.ecs_world.spawn((Point{x:4,y:4}, Renderable{glyph:RenderableGlyph::Jeans as u8, order: RenderOrder::Item}, "Jeans".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Legs}, DefenseBonus{ bonus:0.1}, ToRemove{yes:false}));
        self.ecs_world.insert_one(boots, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Feet});
        self.ecs_world.insert_one(l_jacket, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Torso});
        self.ecs_world.insert_one(jeans, Equipped{ owner: th.to_bits(), slot: EquipmentSlot::Legs});

        let it = self.ecs_world.spawn((Point{x:6,y:7}, Renderable{glyph: RenderableGlyph::Knife as u8, order: RenderOrder::Item}, "Combat knife".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Melee }, MeleeBonus{ bonus: 2}, ToRemove{yes:false}));
        let med = self.ecs_world.spawn((Point{x:5, y:5}, Renderable{glyph: RenderableGlyph::Medkit as u8, order: RenderOrder::Item}, "Medkit".to_string(), Item{}, ToRemove{yes:false}, Consumable{}, ProvidesHealing{heal_amount:5}));
        let boots = self.ecs_world.spawn((Point{x:6, y:18}, Renderable{glyph: RenderableGlyph::Boots as u8, order: RenderOrder::Item}, "Boots".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Feet }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let l_jacket = self.ecs_world.spawn((Point{x:6,y:18}, Renderable{glyph: RenderableGlyph::Jacket as u8, order: RenderOrder::Item}, "Leather jacket".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Torso }, DefenseBonus{ bonus: 0.15 }, ToRemove{yes:false}));
        let jeans = self.ecs_world.spawn((Point{x:6,y:18}, Renderable{glyph: RenderableGlyph::Jeans as u8, order: RenderOrder::Item}, "Jeans".to_string(), Item{}, Equippable{ slot: EquipmentSlot::Legs}, DefenseBonus{ bonus:0.1}, ToRemove{yes:false}));
        
        //debug
        log!("Spawned entities!");
        //log!("{}", &format!("Player stats: {:?}", *state.ecs_world.get::<Attributes>(player).unwrap()));
               
    }

    pub fn spawn_entities_list(&mut self, list_spawns:Vec<(usize, String)>, data: &Vec<NPCPrefab>) {
        for entity in list_spawns.iter() {
            let pos = self.map.idx_xy(entity.0);
            self.spawn(pos.0, pos.1, entity.1.clone(), &data);
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
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &Renderable)>()
        .without::<InBackpack>().without::<Equipped>() //no ref/pointer here!
        .iter() {
            if point.x as usize == x && point.y as usize == y {
                ent = Some(id);
                break;
            }
        }
        return ent;
    }

    pub fn props_list_by_render(&self, render_f: u8) -> Vec<Entity> {
        let mut props = Vec::new();
        //props do not have a name, just a point and render
        for (id, (point, render)) in self.ecs_world.query::<(&Point, &Renderable)>()
        .without::<String>()
        .iter() {
            if render.glyph == render_f {
                props.push(id);
            }
        }
        return props;
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
                //if it's equipped already...
                if self.ecs_world.get::<Equipped>(wantstouse.item).is_ok(){
                    let equipped = self.ecs_world.get::<Equipped>(wantstouse.item).unwrap();
                    let owner = hecs::Entity::from_bits(equipped.owner);
                    if owner == *user {
                        to_unequip.push(wantstouse.item);
                        //if target == *player_entity {
                        game_message(&format!("{{rYou unequip {}.", self.ecs_world.get::<String>(wantstouse.item).unwrap().to_string()));
                    }
                }
                else {
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

    pub fn drop_item(&mut self, user: &Entity, it: &Entity) {
        // The indirection is here to make it possible for non-player Entities to drop items
        //tell the engine that we want to drop the item
        self.ecs_world.insert_one(*user, WantsToDropItem{item:*it});

        //message
        game_message(&format!("{} drops {}", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<String>(*it).unwrap().to_string()));
        //scope to get around borrow checker
        {
            let user_pos = self.map.idx_xy(self.player_position);
            //for NPCs
            //let user_pos = self.ecs_world.get::<Point>(*user).unwrap();
            for (id, (wantstodrop)) in self.ecs_world.query::<(&WantsToDropItem)>().iter(){
                let mut pos = self.ecs_world.get_mut::<Point>(wantstodrop.item).unwrap();
                pos.x = user_pos.0;
                pos.y = user_pos.1;
                //for NPCs
                //pos.x = user_pos.x;
                //pos.y = user_pos.y; 
            }
        }

        self.ecs_world.remove_one::<InBackpack>(*it);
        
    }


    //a very simple test, akin to flipping a coin or throwing a d2
    fn make_test_d2(&self, skill: u32) -> Vec<bool> {
        let mut rolls = Vec::new();
        for _ in 0..10-skill { // exclusive of end
            rolls.push(rand::random()) // generates a boolean
        }
        return rolls
    }

    pub fn attack(&self, target: &Entity) {
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

    pub fn is_player_dead(&self) -> bool {
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

    pub fn end_turn(&mut self) {
        self.get_AI();
        self.remove_dead();
        self.survival_tick();
        self.calendar_time();
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

    fn calendar_time(&mut self) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let mut gs = self.ecs_world.get_mut::<GameState>(entity).unwrap();
                gs.turns += 1;
                //t is a tuple (NaiveTime, i64)
                let t = NaiveTime::from_hms(08, 00, 00).overflowing_add_signed(Duration::seconds(gs.turns));
                let f = t.0.format("%H:%M:%S").to_string();
                game_message(&format!("Time: {}", f));
            },
            None => {},
        }

    }

    fn get_time(&mut self) -> i64 {
        let mut time = 0;

        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let gs = self.ecs_world.get::<GameState>(entity).unwrap();
                time = gs.turns;
            },
            None => {},
        }
        return time;
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
    
    ///-------------------------------------------------------------------------------------
    //AI logic lives here!
    pub fn get_AI(&mut self) {
        let mut wants_path = Vec::new();
        // get the game time once
        let time = self.get_time();
        //log!("{}", &format!("Time: {}", time));

        // we need to borrow mutably (for the movement to happen), so we have to use a Point instead of two usizes (hecs limitation)
        for (id, (ai, point)) in &mut self.ecs_world.query::<(&AI, &mut Point)>()
        .with::<String>()
        .iter()
         {
            //log!("{}", &format!("Got AI {} x {} y {}",  point.x, point.y, self.ecs_world.get::<String>(id).unwrap().to_string())); //just unwrapping isn't enough to format
            
            // exact movement depends on faction
            if self.ecs_world.get::<Faction>(id).is_ok() {
                let fact = self.ecs_world.get::<Faction>(id).unwrap().typ;
                // townsfolk and NOT vendor
                if fact == FactionType::Townsfolk && self.ecs_world.get::<Vendor>(id).is_err() {
                    // 39600 turns (seconds) is equal to 19:00h in chrono
                    if time < 39600 {
                        //random movement
                        let mut x = point.x;
                        let mut y = point.y;
                        //"A single instance is cached per thread and the returned ThreadRng is a reference to this instance" 
                        let mut rng = rand::thread_rng();
                        let move_roll = rng.gen_range(1, 6);
                        match move_roll {
                            1 => x -= 1,
                            2 => x += 1,
                            3 => y -= 1,
                            4 => y += 1,
                            _ => {}
                        }

                        //move
                        let dest_idx = self.map.xy_idx(x, y);
                        if self.map.is_tile_walkable(x,y) && !self.map.is_tile_blocked(dest_idx) {
                            //actually move
                            point.x = x;
                            point.y = y;
                        }
                    } else {
                        // is late, want to find a bed...
                        //log!("{}", &format!("t: {}, wants to find a bed... x {} y {} ", time, point.x, point.y));

                        //if we don't have a bed yet...
                        if self.ecs_world.get::<Path>(id).is_err() {
                            let beds = self.props_list_by_render(RenderableGlyph::Bed as u8);
                            let mut dists = Vec::new();
                            for b in beds {
                                let pt = self.ecs_world.get::<Point>(b).unwrap();
                                let dist = distance2d_chessboard(point.x, point.y, pt.x, pt.y);
                                dists.push((b, dist));
                            }
                            //sort by closest
                            dists.sort_by(|a,b| a.1.cmp(&b.1));
                            
                            let pt = self.ecs_world.get::<Point>(dists[0].0).unwrap();
                            if distance2d_chessboard(point.x, point.y, pt.x, pt.y) > 1 {
                                let path = path_to_target(&mut self.map, point.x as usize, point.y as usize, pt.x as usize, pt.y as usize);
                                
                                //paranoia
                                if path.len() > 1 {
                                    let new_pos = self.map.idx_xy(path[1]);

                                    let mut moved = false;
                                    if !self.map.is_tile_blocked(path[1]) {
                                        let old_idx = self.map.xy_idx(point.x, point.y);
                                        //mark as blocked for pathfinding
                                        self.map.clear_tile_blocked(old_idx);
                                        self.map.set_tile_blocked(path[1]);
    
                                        //actually move
                                        point.x = new_pos.0 as i32;
                                        point.y = new_pos.1 as i32;
    
                                        moved = true;
                                    }
    
    
                                    //don't A* on every turn
                                    wants_path.push((id, path, moved));
                                    // we can't insert while iterating :(
                                    //self.ecs_world.insert_one(id, Path{ steps: path});
                                    //axe the point from path
                                    //self.ecs_world.get_mut::<Path>(id).unwrap().steps.remove(1);
                                }

                            }
                        } else {
                                //log!("We have a path!");
                                //we have a Path
                                let mut path = self.ecs_world.get_mut::<Path>(id).unwrap();
                                //paranoia
                                if path.steps.len() > 2 {
                                    // # 0 is beginning point
                                    let new_pos = self.map.idx_xy(path.steps[1]);

                                    if !self.map.is_tile_blocked(path.steps[1]) {
                                        let old_idx = self.map.xy_idx(point.x, point.y);
                                        //mark as blocked for pathfinding
                                        self.map.clear_tile_blocked(old_idx);
                                        self.map.set_tile_blocked(path.steps[1]);

                                        //actually move
                                        point.x = new_pos.0 as i32;
                                        point.y = new_pos.1 as i32;

                                        //log!("Done a move!");

                                        //axe the point from path
                                        path.steps.remove(1);
                                        //self.ecs_world.get_mut::<Path>(id).unwrap().steps.remove(1);
                                    }

                                }
                            }

                        }
                   

                } else if fact == FactionType::Enemy {
                    //TODO: extract to a function: self.is_visible is the problem here... (map and player position can be passed quite easily)

                    //if the player's immediately next to us, don't run costly A*
                    let player_pos = self.map.idx_xy(self.player_position);
                    //log!("{}", &format!("Player pos x {} y {}", player_pos.0, player_pos.1));
                    if distance2d_chessboard(point.x, point.y, player_pos.0, player_pos.1) < 2 {
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
                                //log!("{}", &format!("new: {} {} player: {} {}", new_pos.0, new_pos.1, player_pos.0, player_pos.1));
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
        
        //postponed stuff to here since we can't add components while iterating
        for w in wants_path {
            self.ecs_world.insert_one(w.0, Path{ steps: w.1});
            if w.2 {
                //axe the point since we already moved by 1 step
                self.ecs_world.get_mut::<Path>(w.0).unwrap().steps.remove(1);
            }

        }
    }

    pub fn debug_console_core(&mut self, input:String) {
        //split by spaces
        let v: Vec<&str> = input.split(' ').collect();
        //debug
        log!("{}", &format!("{:?}", v));
        match v[0] {
            "spawn" => { 
                log!("Debug console entered: spawn"); 
                if v.len() < 2 {
                    log!("Not enough parameters supplied");
                } else {
                    let current_position = self.map.idx_xy(self.player_position);
                    self.spawn(current_position.0+1, current_position.1+1, v[1].to_string(), &DATA.lock().unwrap().npcs)
                }
            },
            _ => { log!("Unknown command entered"); }
        }
    }


} //end of Universe impl