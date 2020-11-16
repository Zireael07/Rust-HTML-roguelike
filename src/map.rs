pub struct Map {
    pub width: u32,
    pub height: u32,
    blocked: Vec<bool>,
    pub revealed_tiles : Vec<bool>,
}

impl Map {
    pub fn new(w:u32, h:u32) -> Map {
        let mut blocked = Vec::new();
        let mut revealed = Vec::new();
        for _i in 0 .. (w*h) {
            blocked.push(false);
            revealed.push(false);
        }
        return Map{width: w, height: h, blocked: blocked, revealed_tiles: revealed};
    }

    //blocked for pathfinding
    pub fn set_tile_blocked(&mut self, idx : i32) {
        self.blocked[idx as usize] = true;
    }

    pub fn clear_tile_blocked(&mut self, idx : i32) {
        self.blocked[idx as usize] = false;
    }

    pub fn is_tile_blocked(&self, idx: i32) -> bool {
        return self.blocked[idx as usize];
    }

    pub fn is_tile_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width as i32-1 || y < 1 || y > self.height as i32-1 { return false; }
        let idx = (y * self.width as i32) + x;
        return !self.is_tile_blocked(idx);
    }
}