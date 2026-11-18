use std::marker::PhantomData;

use js::JsObject;

/// The classes of `./canvas.module.css`.
#[cfg_attr(rust_js, rust_js::css_module)]
pub struct Styles(PhantomData<JsObject>);

impl Styles {
    #[cfg_attr(rust_js, rust_js::link_name = "get canvas-scroll")]
    pub fn canvas_scroll(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get canvas-stage")]
    pub fn canvas_stage(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get canvas-toolbar")]
    pub fn canvas_toolbar(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get zoom-tools")]
    pub fn zoom_tools(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get zoom-value")]
    pub fn zoom_value(&self) -> String {
        unreachable!()
    }
}

unsafe extern "Rust" {
    #[link_name = "./canvas.module.css#default"]
    pub safe static STYLES: &'static Styles;
}
