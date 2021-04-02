//for fetching data files
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use wasm_bindgen::JsCast; // for dyn_into

use serde::{Serialize, Deserialize};

use super::log;
use super::{Universe, RenderableGlyph, AI, Faction, CombatStats,
Item, Equippable, DefenseBonus};

use std::sync::Mutex;


//what it says
#[derive(Deserialize)]
pub struct DataMaster {
    pub npcs : Vec<NPCPrefab>,
    pub items: Vec<ItemPrefab>,
    pub map : MapConfig,
}


#[derive(Serialize, Deserialize)]
pub struct NPCPrefab {
    pub name: String,
    pub renderable: RenderableGlyph,
    pub ai: Option<AI>,
    pub faction: Option<Faction>, 
    pub combat: Option<CombatStats>,
}

#[derive(Serialize, Deserialize)]
pub struct ItemPrefab {
    pub name: String,
    pub renderable: RenderableGlyph,
    pub item: Option<Item>,
    pub equippable: Option<Equippable>,
    pub defense: Option<DefenseBonus>,
}

#[derive(Deserialize, Debug)]
pub struct MapConfig {
    pub width: u32,
    pub height: u32,
    pub octaves: i32,
    pub gain: f32,
    pub lacuna: f32,
    pub frequency: f32,
}

lazy_static! {
    pub static ref DATA: Mutex<DataMaster> = Mutex::new(DataMaster::empty());
}


impl DataMaster {
    pub fn empty() -> DataMaster {
        DataMaster {
            npcs: Vec::new(),
            items: Vec::new(),
            map: MapConfig{width:2, height:2, octaves:1, gain:0.5,lacuna:0.5, frequency:0.5}, //dummy
        }
    }

    pub fn load(&mut self, loaded: DataMaster) {
        //just copy everything over
        self.npcs = loaded.npcs;
        self.items = loaded.items;
        self.map = loaded.map;
    }
}

//async loader based on https://rustwasm.github.io/docs/wasm-bindgen/examples/fetch.html
// returning Universe as a workaround for https://github.com/rustwasm/wasm-bindgen/issues/1858
pub async fn load_datafile(mut state: Universe) -> Universe {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = "./data.ron";

    let request = Request::new_with_str_and_init(&url, &opts).unwrap(); //no ? because we don't return Result

    request
        .headers();
        //.set("Accept", "application/vnd.github.v3+json")?;
        //.unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap(); //?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`, and then to string
    let ron = JsFuture::from(resp.text().unwrap()).await.unwrap().as_string().unwrap(); //?;

    log!("Loaded from rust: {}", &format!("{:?}", ron));

    let data : DataMaster = ron::from_str(&ron).expect("malformed file");
    //debug
    for e in &data.npcs {
        log!("{}", &format!("Ent from prefab: {} {:?} {:?} {:?} {:?}", e.name, e.renderable, e.ai, e.faction, e.combat));
    }
    for e in &data.items {
        log!("{}", &format!("Item from prefab: {} {:?} {:?} {:?} {:?}", e.name, e.renderable, e.item, e.equippable, e.defense));
    }


    DATA.lock().unwrap().load(data);
        
    //log!("{}", &format!("{:?}", data));
    //DATA.lock().unwrap() = data;


    state.game_start(&DATA.lock().unwrap());
    //state.game_start(data);
    //state.spawn_entities(data);

    return state
}