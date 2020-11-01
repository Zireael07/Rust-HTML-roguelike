use super::utils::*;
use super::Universe;

use std::collections::HashMap;

#[allow(dead_code)]
const MAX_DIRECT_PATH_CHECK : f32 = 2048.0;

#[allow(dead_code)]
const MAX_ASTAR_STEPS :i32 = 2048;


fn get_available_neighbors(map: &Universe, idx:i32) -> Vec<(i32, f32)> {
    let mut neighbors : Vec<(i32, f32)> = Vec::new();
    let x = idx % map.width as i32;
    let y = idx / map.width as i32;

    // Cardinal directions
    if map.is_tile_valid(x-1, y) { neighbors.push((idx-1, 1.0)) };
    if map.is_tile_valid(x+1, y) { neighbors.push((idx+1, 1.0)) };
    if map.is_tile_valid(x, y-1) { neighbors.push((idx-map.width as i32, 1.0)) };
    if map.is_tile_valid(x, y+1) { neighbors.push((idx+map.width as i32, 1.0)) };

    // Diagonals
    if map.is_tile_valid(x-1, y-1) { neighbors.push(((idx-map.width as i32)-1, 1.4)); }
    if map.is_tile_valid(x+1, y-1) { neighbors.push(((idx-map.width as i32)+1, 1.4)); }
    if map.is_tile_valid(x-1, y+1) { neighbors.push(((idx+map.width as i32)-1, 1.4)); }
    if map.is_tile_valid(x+1, y+1) { neighbors.push(((idx+map.width as i32)+1, 1.4)); }

    return neighbors;
}

#[allow(dead_code)]
pub fn a_star_search(start:i32, end:i32, map: &Universe) -> NavigationPath {
    let mut searcher = AStar::new(start, end);
    return searcher.search(map);
}

#[allow(dead_code)]
pub struct NavigationPath {
    pub destination: i32,
    pub success: bool,
    pub steps: Vec<i32>
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct Node {
    idx : i32,
    f : f32,
    g : f32,
    h : f32
}

#[allow(dead_code)]
impl NavigationPath {
    fn new() -> NavigationPath {
        return NavigationPath{destination:0, success:false, steps: Vec::new() };
    }
}

#[allow(dead_code)]
pub struct AStar {
    start: i32,
    end : i32,
    open_list: Vec<Node>,
    closed_list: HashMap<i32, f32>,
    parents: HashMap<i32, i32>,
    step_counter: i32
}

impl AStar {
    pub fn new(start : i32, end: i32) -> AStar {
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

    fn distance_to_end(&self, idx :i32, map: &Universe) -> f32 {
        return distance2d(&Point{x:idx_xy(idx as usize).0, y:idx_xy(idx as usize).1}, &Point{x:idx_xy(self.end as usize).0, y:idx_xy(self.end as usize).1});
    }

    fn add_node(&mut self, q:Node, idx:i32, cost:f32, map: &Universe) -> bool {
        // Did we reach our goal?
        if idx == self.end {
            self.parents.insert(idx, q.idx);
            return true;
        } else {
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

            return false;
        }
    }

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

    fn search(&mut self, map: &Universe) -> NavigationPath {
        let result = NavigationPath::new();
        while self.open_list.len() != 0 && self.step_counter < MAX_ASTAR_STEPS {
            self.step_counter += 1;

            // Pop Q off of the list
            let q = self.open_list[0];
            self.open_list.remove(0);

            // Generate neighbors
            let neighbors = get_available_neighbors(map, q.idx);

            for n in neighbors.iter() {
                if self.add_node(q, n.0, n.1 + q.f, map) {
                    let success = self.found_it();
                    return success;
                }
            }

            if self.closed_list.contains_key(&q.idx) { self.closed_list.remove(&q.idx); }
            self.closed_list.insert(q.idx, q.f);
            self.open_list.sort_by(|a,b| a.f.partial_cmp(&b.f).unwrap());            
        }
        return result;
    }
}