use std::any::type_name;
use wasm_bindgen::prelude::*;
use web_sys::{console, Element};

fn window() -> web_sys::Window {
    web_sys::window().expect("get window should never fail in a browser")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("really kinda expecting a document to also exist")
}

fn get_node_by_id<T: JsCast>(id: &str) -> Result<T, DomError> {
    match document().get_element_by_id(id) {
        Some(elem) => elem.dyn_into::<T>().map_err(|elem| DomError::WrongType {
            expected: type_name::<T>().to_string(),
            got: elem,
        }),
        None => return Err(DomError::NoSuchElementId(id.to_string())),
    }
}

#[derive(Debug)]
enum DomError {
    NoSuchElementId(String),
    #[allow(dead_code)]
    WrongType { expected: String, got: Element },
}

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let app: Element = get_node_by_id("app").expect("app ought to exist");

    console::log_2(&JsValue::from_str("Found root:"), &app);

    Ok(())
}

#[wasm_bindgen()]
pub fn hello_world() {
    get_node_by_id::<Element>("app")
        .expect("app ought to exist")
        .append_with_str_1("Rust says: Hello World! *(^_^)*")
        .expect("failed to append");
}
