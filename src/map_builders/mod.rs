use super::{Map, Cell};
use super::fastnoise;

mod noise_map;
use noise_map::NoiseMapBuilder;

//Rust's interface
trait MapBuilder {
    fn build() -> Map;
}

//Public functions for separate builders
pub fn build_noise_map() -> Map {
    NoiseMapBuilder::build()
} 