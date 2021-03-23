use super::log;
use super::{Universe, 
    Point, Player, Renderable, GameState, Needs, Money, 
    AI, Vendor, CombatStats, Faction, 
    Item, InBackpack, Consumable, ProvidesHealing, ProvidesFood, ProvidesQuench, Equippable, MeleeBonus, Equipped};

//save/load
use serde::{Serialize, Deserialize};
use serde_json::json;

// what it says on the tin
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    entity: u64, //because Entity cannot be serialized by serde
    name: String,
    point: Option<Point>,
    render: Option<Renderable>,
    player: Option<Player>,
    gamestate: Option<GameState>,
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

///---------------------------------------------------------------------------------------------------
//save/load
pub fn save_game(u: &Universe) -> String {
    log!("Saving game...");
    //iterate over all entities
    let entities = u.ecs_world.iter().map(|(id, _)| id).collect::<Vec<_>>();
    let mut save_datas : Vec<SaveData> = Vec::new();

    for e in entities {
        //note to self: JSON macro doesn't work with conditionals
        //so we need an intermediate struct
        let mut saved = SaveData{
            entity: e.to_bits(),
            point: None,
            render: None,
            name: "".to_string(), //because props don't have names //u.ecs_world.get::<String>(e).unwrap().to_string(),
            player: None,
            gamestate: None,
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
        if u.ecs_world.get::<Point>(e).is_ok() {
            saved.point = Some(*u.ecs_world.get::<Point>(e).unwrap()); //they all need to be dereferenced
        }
        if u.ecs_world.get::<Renderable>(e).is_ok() {
            saved.render = Some(*u.ecs_world.get::<Renderable>(e).unwrap());
        }
        //props don't have names
        if u.ecs_world.get::<String>(e).is_ok(){
            saved.name = u.ecs_world.get::<String>(e).unwrap().to_string()
        }
        //those aren't guaranteed
        if u.ecs_world.get::<Player>(e).is_ok() {
            //log!("{:?} is player", e);
            saved.player = Some(*u.ecs_world.get::<Player>(e).unwrap());
            //save player position
            let current_position = u.map.idx_xy(u.player_position);
            saved.point = Some(Point{x:current_position.0, y:current_position.1});
        }
        if u.ecs_world.get::<GameState>(e).is_ok(){
            saved.gamestate = Some(*u.ecs_world.get::<GameState>(e).unwrap());
        }
        if u.ecs_world.get::<AI>(e).is_ok(){
            saved.ai = Some(*u.ecs_world.get::<AI>(e).unwrap());
        }
        if u.ecs_world.get::<Needs>(e).is_ok(){
            saved.needs = Some(*u.ecs_world.get::<Needs>(e).unwrap());
        }
        if u.ecs_world.get::<Money>(e).is_ok(){
            saved.money = Some(*u.ecs_world.get::<Money>(e).unwrap());
        }
        if u.ecs_world.get::<Faction>(e).is_ok(){
            saved.faction = Some(*u.ecs_world.get::<Faction>(e).unwrap());
        }
        if u.ecs_world.get::<Vendor>(e).is_ok(){
            saved.vendor = Some(*u.ecs_world.get::<Vendor>(e).unwrap());
        }
        if u.ecs_world.get::<CombatStats>(e).is_ok(){
            saved.combat = Some(*u.ecs_world.get::<CombatStats>(e).unwrap());
        }
        if u.ecs_world.get::<Item>(e).is_ok() {
            saved.item = Some(*u.ecs_world.get::<Item>(e).unwrap());
        }
        if u.ecs_world.get::<InBackpack>(e).is_ok() {
            saved.backpack = Some(*u.ecs_world.get::<InBackpack>(e).unwrap());
        }
        if u.ecs_world.get::<Consumable>(e).is_ok() {
            saved.consumable = Some(*u.ecs_world.get::<Consumable>(e).unwrap());
        }
        if u.ecs_world.get::<ProvidesHealing>(e).is_ok(){
            saved.heals = Some(*u.ecs_world.get::<ProvidesHealing>(e).unwrap());
        }
        if u.ecs_world.get::<ProvidesFood>(e).is_ok(){
            saved.food = Some(*u.ecs_world.get::<ProvidesFood>(e).unwrap());
        }
        if u.ecs_world.get::<ProvidesQuench>(e).is_ok(){
            saved.quench = Some(*u.ecs_world.get::<ProvidesQuench>(e).unwrap());
        }
        if u.ecs_world.get::<Equippable>(e).is_ok() {
            saved.equippable = Some(*u.ecs_world.get::<Equippable>(e).unwrap());
        }
        if u.ecs_world.get::<MeleeBonus>(e).is_ok(){
            saved.meleebonus = Some(*u.ecs_world.get::<MeleeBonus>(e).unwrap());
        }
        if u.ecs_world.get::<Equipped>(e).is_ok() {
            saved.equip = Some(*u.ecs_world.get::<Equipped>(e).unwrap()); 
        }

        save_datas.push(saved);
    }
    //'r' stands for Result
    let json_r = serde_json::to_string(&save_datas);
    log!("JSON: {:?} ", json_r);


    //map data
    let json_r2 = serde_json::to_string(&u.map);
    log!("JSON 2: {:?}", json_r2);

    //log!("{}", &format!("{}", serde_json::to_string(&u.player_position).unwrap()));
    // extract String from Result
    if json_r.is_ok() && json_r2.is_ok() {
        //hack because we can't return a tuple or Vec<> of Strings
        return json_r.unwrap() + " \nmap:" + &json_r2.unwrap();
    } else {
        return "".to_string();
    }
}

pub fn load_save(u: &mut Universe, data: String) {
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
                u.player_position = u.map.xy_idx(point.x, point.y);
            }
            if e.gamestate.is_some(){
                builder.add(e.gamestate.unwrap());
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
            u.ecs_world.spawn_at(ent, builder.build());
        }

        //let current_position = u.map.idx_xy(u.player_position);
        // // refresh FOV
        // u.fov_data.clear_fov(); // compute_fov does not clear the existing fov
        // u.fov.compute_fov(&mut u.fov_data, current_position.0 as usize, current_position.1 as usize, 6, true);
    }

    let res =  serde_json::from_str(&split[1]);
    if res.is_ok() {
        let mapa = res.unwrap();
        u.map = mapa;
    }
}