use super::{Map, Cell, Rect};
use super::fastnoise;

mod noise_map;
use noise_map::NoiseMapBuilder;

mod bsp_town;
use bsp_town::BSPTownBuilder;

//Rust's interface
pub trait MapBuilder {
    fn build_map(&mut self) -> Map;
}

//Factory function for builder
pub fn random_builder() -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slightly random
    Box::new(BSPTownBuilder::new())
}

//Public functions for separate builders
// pub fn build_noise_map() -> Map {
//     NoiseMapBuilder::build()
// } 

// pub fn build_town_map() -> Map {
//     BSPTownBuilder::build()
// }