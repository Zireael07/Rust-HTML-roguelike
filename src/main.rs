extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//the websys Canvas bindings uses it
use wasm_bindgen::JsCast; // for dyn_into
use std::f64;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn main() {
 let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    log!("We have a context {:?}", context);

    //clear
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("black"));
    context.fill_rect(0.0, 0.0, 800.0, 600.0);
}


// Auto-starts on page load
//start section of the executable may not literally point to main
#[wasm_bindgen(start)]
pub fn start() {
   main()
} 
