//for fetching data files
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use wasm_bindgen::JsCast; // for dyn_into

use serde::{Serialize, Deserialize};

use super::log;
use super::{Universe, RenderableGlyph, AI, Faction, CombatStats};

use std::sync::Mutex;


//what it says
#[derive(Deserialize)]
pub struct DataMaster {
    pub npcs : Vec<NPCPrefab>
}


#[derive(Serialize, Deserialize)]
pub struct NPCPrefab {
    pub name: String,
    pub renderable: RenderableGlyph,
    pub ai: Option<AI>,
    pub faction: Option<Faction>, 
    pub combat: Option<CombatStats>,
}

lazy_static! {
    pub static ref DATA: Mutex<DataMaster> = Mutex::new(DataMaster::empty());
}


impl DataMaster {
    pub fn empty() -> DataMaster {
        DataMaster {
            npcs: Vec::new(),
        }
    }

    //pub fn load(&mut self, )
}

//async loader based on https://rustwasm.github.io/docs/wasm-bindgen/examples/fetch.html
// returning Universe as a workaround for https://github.com/rustwasm/wasm-bindgen/issues/1858
pub async fn load_datafile(mut state: Universe) -> Universe {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = "./npcs.ron";

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

    let data : Vec<NPCPrefab> = ron::from_str(&ron).expect("malformed file");
    //debug
    for e in &data {
        log!("{}", &format!("Ent from prefab: {} {:?} {:?} {:?} {:?}", e.name, e.renderable, e.ai, e.faction, e.combat));
    }
        
    //log!("{}", &format!("{:?}", data));
    DATA.lock().unwrap().npcs = data;


    state.game_start(&DATA.lock().unwrap());
    //state.game_start(data);
    //state.spawn_entities(data);

    return state
}