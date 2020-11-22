
use super::{MapBuilder, Map, Cell};
use super::fastnoise::*;

pub struct NoiseMapBuilder {}

impl MapBuilder for NoiseMapBuilder {
    fn build_map(&mut self) -> Map {
        let mut map = Map::new(20,20);
        NoiseMapBuilder::noise_build(&mut map);
        map
    }
}

impl NoiseMapBuilder {
    fn noise_build(map: &mut Map) {
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
                //log!("{}", &format!("Noise: x{}y{} {}", x, y, n));
                if n > 125.0 || n < -125.0 {
                    let idx = map.xy_idx(x,y);
                    map.tiles[idx] = Cell::Wall as u8;
                } else {
                    //state.tiles[xy_idx(x,y)] = Cell:Floor as u8
                }
                //log!("{}", &format!("Tile: x{} y{} {}", x,y, state.tiles[xy_idx(x,y)]));
            }
        }
      


        // Make the boundaries walls
        for x in 0..20 {
            let mut idx = map.xy_idx(x, 0);
            map.tiles[idx] = Cell::Wall as u8;
            idx = map.xy_idx(x, 19);
            map.tiles[idx] = Cell::Wall as u8;
        }
        for y in 0..20 {
            let mut idx = map.xy_idx(0, y); 
            map.tiles[idx] = Cell::Wall as u8;
            idx = map.xy_idx(19, y);
            map.tiles[idx] = Cell::Wall as u8;
        }

        //map
    }
}