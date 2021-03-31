use std::borrow::BorrowMut; //to be able to pass Path from system to function

use super::log;
use super::{game_message, path_to_player, path_to_target,
    Universe,
    AI, Faction, FactionType, Vendor, Asleep, Player,
    Point, Path, Map, RenderableGlyph};

use hecs::Entity;

//RNG
use rand::Rng;

use super::utils::*;

//avoid passing self for the same reason as in pathing functions in universe_private.rs (system has a borrow on self)
fn setup_path_and_step(map: &mut Map, id: Entity, path: Vec<usize>, point: &mut Point, wants_path: &mut Vec<(Entity, Vec<usize>, bool)>) {
    //paranoia
    if path.len() > 1 {
        let new_pos = map.idx_xy(path[1]);

        let mut moved = false;
        if !map.is_tile_blocked(path[1]) {
            let old_idx = map.xy_idx(point.x, point.y);
            //mark as blocked for pathfinding
            map.clear_tile_blocked(old_idx);
            map.set_tile_blocked(path[1]);

            //actually move
            point.x = new_pos.0 as i32;
            point.y = new_pos.1 as i32;

            moved = true;
        }


        //don't A* on every turn
        wants_path.push((id, path, moved));
    }
}

fn move_along_path(map: &mut Map, path: &mut Path, point: &mut Point) {
    // # 0 is beginning point
    let new_pos = map.idx_xy(path.steps[1]);

    if !map.is_tile_blocked(path.steps[1]) {
        let old_idx = map.xy_idx(point.x, point.y);
        //mark as blocked for pathfinding
        map.clear_tile_blocked(old_idx);
        map.set_tile_blocked(path.steps[1]);

        //actually move
        point.x = new_pos.0 as i32;
        point.y = new_pos.1 as i32;

        //axe the point from path
        path.steps.remove(1);
    }
}

impl Universe {
    //AI logic lives here!
    pub fn get_AI(&mut self) {
        let mut wants_path = Vec::new();
        let mut wants_sleep = Vec::new();

        // get the game time once
        //let time = self.get_time();
        let time = self.get_time_of_day();
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
                    // 39600 turns (seconds) is equal to 19:00h in chrono (if we count from 8:00)
                    //28 800 is turns since midnight for 8:00h (game start)
                    if time < 39600+28800 {
                        if time > 25400 {
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
                        } else if time > 21600 { // after 6:00h
                            //log!("Time to get up!");
                            // for some reason, some times are skipped, so we check
                            if self.ecs_world.get::<Asleep>(id).is_ok() {
                                //hack solution: we know the one vendor is in pub
                                for (ent_id, vendor) in self.ecs_world.query::<(&Vendor)>().iter() {
                                    let tg = self.ecs_world.get::<Point>(ent_id).unwrap();
                                    let path = path_to_target(&mut self.map, point.x as usize, point.y as usize, tg.x as usize, tg.y as usize);

                                    //log!("{}", &format!("We have a path to vendor: {:?}", path));
                                    setup_path_and_step(&mut self.map, id, path, point, &mut wants_path);
                                }
                                //we're awake now...
                            } else {
                                if self.ecs_world.get_mut::<Path>(id).is_ok() {
                                    //we have a Path
                                    let mut path = self.ecs_world.get_mut::<Path>(id).unwrap();

                                    //log!("We have a path back to the barkeep!");
                                    //paranoia
                                    if path.steps.len() > 2 {
                                       move_along_path(&mut self.map, path.borrow_mut(), point);
                                    }
                                }
                                
                            }
                           
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
                                setup_path_and_step(&mut self.map, id, path, point, &mut wants_path);

                            }
                        } else {
                                //log!("We have a path!");
                                //we have a Path
                                let mut path = self.ecs_world.get_mut::<Path>(id).unwrap();
                                //paranoia
                                if path.steps.len() > 2 {
                                    //see: https://stackoverflow.com/questions/49841847/passing-the-contents-of-a-refcellmut-t-to-a-function
                                    move_along_path(&mut self.map, path.borrow_mut(), point);                                    
                                } else {
                                    //we're done, mark us as asleep
                                    wants_sleep.push(id);
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
            //if we already have a path, nuke it
            self.ecs_world.remove_one::<Path>(w.0);
            //wanting a path implies not being asleep
            self.ecs_world.remove_one::<Asleep>(w.0);

            self.ecs_world.insert_one(w.0, Path{ steps: w.1});
            if w.2 {
                //axe the point since we already moved by 1 step
                self.ecs_world.get_mut::<Path>(w.0).unwrap().steps.remove(1);
            }

        }
        for id in wants_sleep {
            self.ecs_world.insert_one(id, Asleep{});
        }
    }
}