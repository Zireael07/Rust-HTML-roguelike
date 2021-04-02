use super::{MetaMapBuilder, BuilderMap, Cell, Rect};
use super::data_loader::*;
use super::log; //macro

pub struct RectBuilder {}

impl MetaMapBuilder for RectBuilder {
    fn build_map(&mut self, build_data : &mut BuilderMap, data: &DataMaster)  {
        self.build(build_data);
    }
}

impl RectBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<RectBuilder> {
        Box::new(RectBuilder{})
    }

    fn build(&mut self, build_data : &mut BuilderMap) {
        let floors = self.num_unbroken_floors_columns(build_data);
        let row_floors = self.unbroken_floors_per_row(floors);
        let rect = self.largest_area_rect(row_floors); // x, y, h, w

        //submaps setup
        let mut submaps : Vec<Rect> = Vec::new();
        submaps.push(Rect::new(rect.0, rect.1, rect.3, rect.2));
        build_data.submaps = Some(submaps);
        //console::log(format!("Submaps: {:?}", build_data.submaps));

        //here comes nothing...
        //console::log(format!("Y {:?}-{:?} X:{:?}-{:?}", rect.1, rect.1+rect.2, rect.0, rect.0+rect.3));

        //paranoia
        let max_y = if rect.1+rect.2 < build_data.map.height as i32 { rect.1 + rect.2 } else { build_data.map.height as i32 };
        let max_x = if rect.0 + rect.3 < build_data.map.width as i32 { rect.0 + rect.3 } else { build_data.map.width as i32 };
        for y in rect.1 .. max_y {
            for x in rect.0 .. max_x {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = Cell::Floor as u8;
            }
        }
        //build_data.take_snapshot();

    }

    // step one of finding rectangle of floor in matrix
    // https://stackoverflow.com/a/12387148
    fn num_unbroken_floors_columns(&self, build_data : &mut BuilderMap) -> Vec<Vec<i32>> {
        //first set it to 0
        let mut num_floors = vec![vec![0; build_data.map.height as usize]; build_data.map.width as usize];
        //Rust is weird, ranges are inclusive at the beginning but exclusive at the end
        for x in 0 .. build_data.map.width as usize {
            for y in 0 .. build_data.map.height as usize {
                //paranoia
                let add = if y == 0 { 0 } else { num_floors[x][y-1] };
                //pretty and readable but crashes for y == 0
                //let north = (x + 0, y - 1);
                //console::log(&format!("X,Y {:?},{:?} - num floors north: {:?} ", x, y, add));
                let idx = build_data.map.xy_idx(x as i32, y as i32);
                //Rust's ternary expression
                num_floors[x][y] = if build_data.map.tiles[idx] == Cell::Grass as u8 { 1 + add } else {0};
            }
        }

        //console::log(&format!("{:?}", num_floors));
        num_floors
    }

    //Parse it nicely for every y row
    fn unbroken_floors_per_row(&self, floors: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        let mut ret : Vec<Vec<i32>> = Vec::new();
        for y in 0 .. floors[0].len() {
            let mut row : Vec<i32> = Vec::new();
            for x in 0 .. floors.len() {
                row.push(floors[x][y]);
            }
            ret.push(row);
            //console::log(&format!("{:?}", row));
        }
        //console::log(&format!("{:?}", ret));
        return ret;
    }

    //step two of finding rectangle of floor in matrix
    // https://stackoverflow.com/a/12387148
    fn largest_area_rect(&self, floor_rows : Vec<Vec<i32>>) -> (i32, i32, i32, i32) {
        let mut rects = Vec::<(i32,i32,i32,i32,i32)>::new();

        //reverse order
        let mut y = floor_rows.len();
        for row in floor_rows.iter().rev() {
            y -= 1;
            rects.push(self.max_rectangle_histogram(row.to_vec(), y as i32));
        }
        //console::log(&format!("Rects: {:?}", rects));

        //sort (cmp works by reference)
        rects.sort_by(|a, b| a.0.cmp(&b.0));
        //console::log(&format!("Rects sorted: {:?}", rects));

        //it seems to work on ascending order by default, so get the last
        //console::log(&format!("Biggest rect: {:?}", rects[rects.len()-1]));
        let biggest = rects[rects.len()-1];
        //console::log(&format!("x {:?} y {:?} h {:?} w {:?}  = area: {:?} ", biggest.3, biggest.4, biggest.1, biggest.2, biggest.0));
        (biggest.3, biggest.4, biggest.1, biggest.2)
    }

    //https://codereview.stackexchange.com/questions/197112/largest-rectangle-in-histogram-in-rust?rq=1
    fn max_rectangle_histogram(&self, histogram: Vec<i32>, id: i32) -> (i32, i32, i32, i32, i32) {
        let mut stack: Vec<usize> = Vec::new();
        // to work around the fact that we're modifying, and we can't really make the histogram mut here for some reason...
        let mut v = histogram.clone();
        // populate stack with 0 - (index of first element in 'v') to avoid checking for empty stack
        stack.push(0);
        // insert -1 from both sides so that we don't have to test for corner cases
        // trick described in e.g. http://shaofanlai.com/post/85
        v.insert(0, -1);
        v.push(-1);
        let mut max_area = 0;
        //dummy
        let mut answer = (0,0,0,0,0);
        for (i, h) in v.iter().enumerate() {
            let idx = i as i32;
            // If this bar is higher than the previous, push it to stack 
            if h > &v[stack.last().unwrap().clone()] {
                stack.push(i);
            } else {
                // If this bar is lower, pop the bar from stack
                while h < &v[stack.last().unwrap().clone()] {
                    let last_bar = v[stack.pop().unwrap().clone()];
                    //unlike the more popular version, we don't need to have a case for empty stack here
                    let width = idx - 1 - stack.last().unwrap().clone() as i32;
                    let area = last_bar * width;

                    if area > max_area {
                        max_area = area;
                        // answer is area, height, width, x, id = y (y last because it comes from outside the calculation itself)
                        // this algo is bottom-up, so deduce height from y to get the top
                        //it goes left->right, so the i (position in histogram is the right end)
                        answer = (area, last_bar, width, i as i32-width, id-last_bar); 

                    }
                }
                stack.push(i);
            }
        }
        //log!("{}", &format!("x {:?} y {:?} h {:?} w {:?}  = area: {:?} ", answer.3, answer.4, answer.1, answer.2, answer.0));
        answer
    }

}