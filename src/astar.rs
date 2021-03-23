use super::utils::*;
use super::Map;

use std::collections::HashMap;


#[allow(dead_code)]
const MAX_ASTAR_STEPS :usize = 65536;


fn neighbor_idx(map: &Map, sx: i32, sy: i32, delta_x: i32, delta_y: i32) -> usize {
    let destination = (sx + delta_x, sy + delta_y);
    let idx = map.xy_idx(destination.0, destination.1);
    return idx;
}

fn get_available_neighbors(map: &Map, idx:usize) -> Vec<(usize, f32)> {
    let mut neighbors : Vec<(usize, f32)> = Vec::new();
    let x = idx as i32 % map.width as i32;
    let y = idx as i32 / map.width as i32;

    // Cardinal directions
    if map.is_in_bounds(x-1, y) && map.is_tile_walkable(x-1, y) { 
        let idx = neighbor_idx(map, x,y, -1,0);
        neighbors.push((idx, 1.0)) 
    };
    if map.is_in_bounds(x+1, y) && map.is_tile_walkable(x+1, y) { 
        let idx = neighbor_idx(map, x,y, 1, 0);
        neighbors.push((idx, 1.0)) 
    };
    if map.is_in_bounds(x, y-1) && map.is_tile_walkable(x, y-1) { 
        let idx = neighbor_idx(map, x,y, 0, -1);
        neighbors.push((idx, 1.0)) 
    };
    if map.is_in_bounds(x, y+1) && map.is_tile_walkable(x, y+1) { 
        let idx = neighbor_idx(map, x,y, 0, 1);
        neighbors.push((idx, 1.0)) 
    };

    // Diagonals
    if map.is_in_bounds(x-1, y-1) && map.is_tile_walkable(x-1, y-1) { 
        let idx = neighbor_idx(map, x,y, -1, -1);
        neighbors.push((idx, 1.4)); 
    }
    if map.is_in_bounds(x+1, y-1) && map.is_tile_walkable(x+1, y-1) { 
        let idx = neighbor_idx(map, x,y, 1, -1);
        neighbors.push((idx, 1.4)); 
    }
    if map.is_in_bounds(x-1, y+1) && map.is_tile_walkable(x-1, y+1) { 
        let idx = neighbor_idx(map, x,y, -1, 1);
        neighbors.push((idx, 1.4)); 
    }
    if map.is_in_bounds(x+1, y+1) && map.is_tile_walkable(x+1, y+1) { 
        let idx = neighbor_idx(map, x, y, 1, 1);
        neighbors.push((idx, 1.4)); 
    }

    return neighbors;
}

#[allow(dead_code)]
pub fn a_star_search(start:usize, end:usize, map: &Map) -> NavigationPath {
    let mut searcher = AStar::new(start, end);
    return searcher.search(map);
}

#[allow(dead_code)]
pub struct NavigationPath {
    pub destination: usize,
    pub success: bool,
    pub steps: Vec<usize>
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct Node {
    idx : usize,
    f : f32,
    g : f32,
    h : f32
}

#[allow(dead_code)]
impl NavigationPath {
    /// Makes a new (empty) NavigationPath
    fn new() -> NavigationPath {
        return NavigationPath{destination:0, success:false, steps: Vec::new() };
    }
}

#[allow(dead_code)]
/// Private structure for calculating an A-Star navigation path.
pub struct AStar {
    start: usize,
    end : usize,
    open_list: Vec<Node>,
    closed_list: HashMap<usize, f32>,
    parents: HashMap<usize, usize>,
    step_counter: usize
}

impl AStar {
    /// Creates a new path, with specified starting and ending indices.
    pub fn new(start : usize, end: usize) -> AStar {
        let mut open_list : Vec<Node> = Vec::new();
        open_list.push(Node{ idx : start, f: 0.0, g: 0.0, h: 0.0 });

        return AStar{ start: start, 
            end : end, 
            open_list : open_list, 
            parents: HashMap::new(), 
            closed_list: HashMap::new(),
            step_counter: 0
        };
    }

    fn distance_to_end(&self, idx :usize, map: &Map) -> f32 {
        return distance2d(&Point{x:map.idx_xy(idx).0, y:map.idx_xy(idx).1}, &Point{x:map.idx_xy(self.end).0, y:map.idx_xy(self.end).1});
    }

    fn add_node(&mut self, q:Node, idx:usize, cost:f32, map: &Map) {
        // Did we reach our goal?
        let distance = self.distance_to_end(idx, map);
        let s = Node{ idx:idx, f:distance + cost, g:cost, h:distance };

        // If a node with the same position is in the open list with a lower f, skip add
        let mut should_add = true;
        for e in self.open_list.iter() {
            if e.f < s.f && e.idx == idx { 
                should_add = false; 
            }
        }

        // If a node with the same position is in the closed list, with a lower f, skip add
        if should_add && self.closed_list.contains_key(&idx) && self.closed_list[&idx] < s.f { 
            should_add = false; 
        }

        if should_add {
            self.open_list.push(s);
            self.parents.insert(idx, q.idx);
        }

    }

    /// Helper function to unwrap a path once we've found the end-point.
    fn found_it(&self) -> NavigationPath {
        let mut result = NavigationPath::new();
        result.success = true;
        result.destination = self.end;

        result.steps.push(self.end);
        let mut current = self.end;
        while current != self.start {
            let parent = self.parents[&current];
            result.steps.insert(0, parent); 
            current = parent;
        }

        return result;
    }

    /// Performs an A-Star search
    fn search(&mut self, map: &Map) -> NavigationPath {
        let result = NavigationPath::new();
        while self.open_list.len() != 0 && self.step_counter < MAX_ASTAR_STEPS {
            self.step_counter += 1;

            // Pop Q off of the list
            let q = self.open_list[0];
            self.open_list.remove(0);

            if q.idx == self.end {
                let success = self.found_it();
                return success;
            }

            // Generate neighbors
            get_available_neighbors(map, q.idx)
                .iter()
                .for_each(|s| self.add_node(q, s.0, s.1 + q.f, map));

            if self.closed_list.contains_key(&q.idx) { self.closed_list.remove(&q.idx); }
            self.closed_list.insert(q.idx, q.f);
            self.open_list.sort_by(|a,b| a.f.partial_cmp(&b.f).unwrap());            
        }
        return result;
    }
}