use super::{InitialMapBuilder, MetaMapBuilder, BuilderMap, Map, Cell, Rect};
use super::log; //macro
//RNG
use rand::Rng;

const MIN_ROOM_SIZE : i32 = 6; //8

pub struct BSPTownBuilder {
    //map: Map,
    rooms: Vec<Rect>,
    rects: Vec<Rect>
}

impl InitialMapBuilder for BSPTownBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, build_data : &mut BuilderMap) {
        self.build(build_data);
    }
}

impl MetaMapBuilder for BSPTownBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, build_data : &mut BuilderMap) {
        //meta version panics if no submaps
        let submaps : Vec<Rect>;
        if let Some(submaps_builder) = &build_data.submaps {
            submaps = submaps_builder.clone();
        } else {
            panic!("Using BSP town as meta requires a builder with submap structures");
        }

        self.build(build_data);
    }
}

impl BSPTownBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<BSPTownBuilder> {
        Box::new(BSPTownBuilder{
            //map : Map::new(20,20),
            rooms: Vec::new(),
            rects: Vec::new()
        })
    }

    fn build(&mut self, build_data : &mut BuilderMap) {
        let mut rooms : Vec<Rect> = Vec::new();

        //we work with submap bounds if we have them, else we work with the whole map
        let mut submaps : Vec<Rect> = Vec::new();
        if let Some(submaps_builder) = &build_data.submaps {
            submaps = submaps_builder.clone();
        }

        let mut sx = 1;
        let mut sy = 1;
        let mut endx = build_data.map.width as i32-1;
        let mut endy = build_data.map.height as i32-1;

        if submaps.len() > 0{
            sx = submaps[0].x1;
            sy = submaps[0].y1;
            endx = submaps[0].x2;
            endy = submaps[0].y2;

        }

        //fill with floors
        for y in sy..endy {
            for x in sx..endx {
                let idx = build_data.map.xy_idx(x as i32, y as i32);
                build_data.map.tiles[idx] = Cell::Floor as u8;
            }
        }


        //place walls around
        //Rust is weird, ranges are inclusive at the beginning but exclusive at the end
        // for x in 0 ..build_data.map.width{
        //     let mut idx = build_data.map.xy_idx(x as i32, 0);
        //     build_data.map.tiles[idx] = Cell::Wall as u8;
        //     idx = build_data.map.xy_idx(x as i32, build_data.map.height as i32-1);
        //     build_data.map.tiles[idx] = Cell::Wall as u8;
        // }
        // for y in 0 ..build_data.map.height{
        //     let mut idx = build_data.map.xy_idx(0, y as i32);
        //     build_data.map.tiles[idx] = Cell::Wall as u8;
        //     idx = build_data.map.xy_idx(build_data.map.width as i32-1, y as i32);
        //     build_data.map.tiles[idx] = Cell::Wall as u8;
        // }

        //self.take_snapshot();

        //BSP now
        self.rects.clear();
        self.rects.push( Rect::new(1, 1, build_data.map.width as i32-2, build_data.map.height as i32-2) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect();

            //stop too small
            let rect_width = i32::abs(rect.x1 - rect.x2);
            let rect_height = i32::abs(rect.y1 - rect.y2);
            if rect_width > MIN_ROOM_SIZE && rect_height > MIN_ROOM_SIZE { 
                let candidate = self.get_random_sub_rect(rect);
                //log!("{}", format!("rect candidate: {:?}", candidate));

                if self.is_possible(candidate, &build_data, &rooms) {
                    rooms.push(candidate);
                    self.add_subrects(rect);
                    //buildings added further on
                }
            }

            n_rooms += 1;
        }



        //let rooms_copy = self.rects.clone();
        let rooms_copy = rooms.clone();
        for r in rooms_copy.iter() {
            let room = *r;
            //rooms.push(room);
            for y in room.y1 .. room.y2 {
                for x in room.x1 .. room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0 && idx < ((build_data.map.width * build_data.map.height)-1) as usize {
                        build_data.map.tiles[idx] = Cell::Wall as u8;
                    }
                }
            }
            //self.take_snapshot();

            for y in room.y1+1 .. room.y2-1 {
                for x in room.x1+1 .. room.x2-1 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0 && idx < ((build_data.map.width * build_data.map.height)-1) as usize {
                        build_data.map.tiles[idx] = Cell::Floor as u8;
                    }
                }
            }
            //self.take_snapshot();

            //build doors
            let cent = room.center();
            let mut rng = rand::thread_rng();
            let door_direction = rng.gen_range(1, 4);
            match door_direction {
                1 => { 
                    let idx = build_data.map.xy_idx(cent.0, room.y1); //north
                    build_data.map.tiles[idx] = Cell::Floor as u8;
                }
                2 => { 
                    let idx = build_data.map.xy_idx(cent.0, room.y2-1); //south
                    build_data.map.tiles[idx] = Cell::Floor as u8;
                }
                3 => { 
                    let idx = build_data.map.xy_idx(room.x1, cent.1); //west
                    build_data.map.tiles[idx] = Cell::Floor as u8;
                }
                _ => { 
                    let idx = build_data.map.xy_idx(room.x2-1, cent.1); //east
                    build_data.map.tiles[idx] = Cell::Floor as u8;
                }
            }
        }
    }

    //taken from BSP dungeon...
    //BSP subdivision happens here
    fn add_subrects(&mut self, rect : Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects.push(Rect::new( rect.x1, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1, rect.y1 + half_height, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1 + half_height, half_width, half_height ));
    }

    //helpers
    fn get_random_rect(&mut self) -> Rect {
        if self.rects.len() == 1 { return self.rects[0]; }
        let mut rng = rand::thread_rng();
        let idx = (rng.gen_range(1, self.rects.len() as i32)-1) as usize; 
        //let idx = (rng.roll_dice(1, self.rects.len() as i32)-1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect : Rect) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        //let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) + 1;
        //let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10))-1) + 1;
        let mut rng = rand::thread_rng();
        
        //let w = rng.gen_range(4,6);
        //let h = rng.gen_range(4,6);
        
        let w = rng.gen_range(6,10);
        let h = rng.gen_range(6,10);
        //let w = rng.roll_dice(2,4)+4;
        //let h = rng.roll_dice(2,4)+4;

        //offset
        result.x1 += rng.gen_range(1,3); //8
        result.y1 += rng.gen_range(1,3);
        //result.x1 += rng.roll_dice(2, 4);
        //result.y1 += rng.roll_dice(2, 4);
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect : Rect, build_data : &BuilderMap, rooms: &Vec<Rect>) -> bool {
        //expanding prevents overlapping rooms
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        let mut can_build = true;

        for r in rooms.iter() {
            if r.intersect(&rect) { 
                can_build = false; 
                //console::log(&format!("Candidate {:?} overlaps a room {:?}", rect, r));
            }
        }

        for y in expanded.y1 ..= expanded.y2 {
            for x in expanded.x1 ..= expanded.x2 {
                if x > build_data.map.width as i32-2 { can_build = false; }
                if y > build_data.map.height as i32-2 { can_build = false; }
                if x < 1 { can_build = false; }
                if y < 1 { can_build = false; }
                if can_build {
                    let idx = build_data.map.xy_idx(x, y);
                    if build_data.map.tiles[idx] != Cell::Floor as u8 { //key change
                        //console::log(&format!("Candidate {:?} failed the tile check!", rect));
                        can_build = false; 
                    }
                }
            }
        }

        can_build
    }
}