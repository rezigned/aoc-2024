mod day6;

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use day6::App;

fn main() {
    console_error_panic_hook::set_once();

    let game = document()
        .get_element_by_id("game")
        .unwrap()
        .unchecked_into();

    leptos::mount::mount_to(game, App).forget();
}
