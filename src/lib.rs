pub mod api;
pub mod app;
pub mod components;
pub mod models;
pub mod pages;
pub mod stores;
pub mod types;
pub mod utils;
pub mod agent;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "hydrate")] {
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen(start)]
        pub fn hydrate() {
            use crate::app::App;
            console_error_panic_hook::set_once();
            leptos::mount::hydrate_body(App);
        }
    }
}

// Re-export app component for use in main
pub use app::App;
