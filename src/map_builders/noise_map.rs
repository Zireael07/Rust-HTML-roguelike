
use super::{InitialMapBuilder, BuilderMap, Map, Cell};
use super::fastnoise::*;
use super::log; //macro

pub struct NoiseMapBuilder {}

impl InitialMapBuilder for NoiseMapBuilder {
    fn build_map(&mut self, build_data : &mut BuilderMap)  {
        //let mut map = Map::new(20,20);
        self.noise_build(build_data);
    }
}

impl NoiseMapBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<NoiseMapBuilder> {
        Box::new(NoiseMapBuilder{})
    }

    fn noise_build(&mut self, build_data : &mut BuilderMap) {
        //noise
        //generate noise
        let mut rng = rand::thread_rng();
        let mut noise = FastNoise::seeded(10001 as u64);
        //let mut noise = FastNoise::seeded(rng.gen_range(1, 65537) as u64);
        noise.set_noise_type(NoiseType::SimplexFractal);
        noise.set_fractal_type(FractalType::FBM);
        
        //for a large map
        noise.set_fractal_octaves(5);
        noise.set_fractal_gain(0.6);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(2.0);

        //for tiny 20x20 map
        //noise.set_fractal_octaves(1);
        //noise.set_fractal_gain(0.5);
        //noise.set_fractal_lacunarity(2.0);
        //noise.set_frequency(0.085);
        //noise.set_frequency(0.45);

        for x in 0..build_data.map.width-1 {
            for y in 0..build_data.map.height-1 {
                let mut n = noise.get_noise(x as f32, y as f32);
                n = n*255 as f32; //because defaults are vanishingly small
                //log!("{}", &format!("Noise: x{}y{} {}", x, y, n));
                let idx = build_data.map.xy_idx(x as i32,y as i32);
                // range from -255 to +255
                //for small map
                //if n > 125.0 || n < -125.0 {
                if n > 140.0 {    
                    build_data.map.tiles[idx] = Cell::Tree as u8;
                } else {
                    build_data.map.tiles[idx] = Cell::Grass as u8;
                }
                //log!("{}", &format!("Tile: x{} y{} {}", x,y, state.tiles[xy_idx(x,y)]));
            }
        }
      


        // Make the boundaries walls
        for x in 0..build_data.map.width-1 {
            let mut idx = build_data.map.xy_idx(x as i32, 0);
            build_data.map.tiles[idx] = Cell::Wall as u8;
            idx = build_data.map.xy_idx(x as i32, build_data.map.height as i32-1);
            build_data.map.tiles[idx] = Cell::Wall as u8;
        }
        for y in 0..build_data.map.height-1 {
            let mut idx = build_data.map.xy_idx(0, y as i32); 
            build_data.map.tiles[idx] = Cell::Wall as u8;
            idx = build_data.map.xy_idx(build_data.map.width as i32-1, y as i32);
            build_data.map.tiles[idx] = Cell::Wall as u8;
        }

        //map
    }
}