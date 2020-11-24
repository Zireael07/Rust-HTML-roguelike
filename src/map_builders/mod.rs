use super::{Map, Cell, Rect};
use super::fastnoise;

mod noise_map;
use noise_map::NoiseMapBuilder;

mod bsp_town;
use bsp_town::BSPTownBuilder;


pub struct BuilderMap {
    pub map : Map,
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data : BuilderMap
}

impl BuilderChain {
    pub fn new(width: i32, height: i32) -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            build_data : BuilderMap {
                map: Map::new(width as u32, height as u32),
            }
        }
    }

    pub fn start_with(&mut self, starter : Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder.")
        };
    }

    //for chaining metabuilders
    pub fn with(&mut self, metabuilder : Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder);
    }

    pub fn build_map(&mut self) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(&mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(&mut self.build_data);
        }
    }
}

//Rust's interface - unfortunately, no variables allowed here!
pub trait MapBuilder {
    fn build_map(&mut self);
    //fn get_map(&mut self) -> Map;
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, build_data : &mut BuilderMap);
}

pub trait MetaMapBuilder {    
    fn build_map(&mut self, build_data : &mut BuilderMap);
}

//Factory function for builder
pub fn random_builder(width: i32, height: i32) -> BuilderChain {
    let mut builder = BuilderChain::new(width, height);
    //builder.start_with(BSPTownBuilder::new());
    builder.start_with(NoiseMapBuilder::new());
    builder
}