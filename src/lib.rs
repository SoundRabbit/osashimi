extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

mod debug;
mod env;
pub mod html;
pub mod kagura;
mod libs;
mod state;

pub use html::component;
pub use html::Html;
pub use kagura::Kagura;

pub mod prelude {
    pub use crate::html::component::{Constructor, PrepackedComponent, Render, Update};
    pub use crate::html::{self, Attributes, Component, Events};
    pub use crate::Html;
    pub use crate::Kagura;
}
