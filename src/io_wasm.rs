use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[allow(unused)]
    pub fn alert(s: &str);

    #[allow(unused)]
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    pub fn poll_restart() -> bool;
    pub fn get_pieces_string() -> String;
    pub fn get_promotions_string() -> String;
    pub fn get_bottom_side() -> bool;
}
