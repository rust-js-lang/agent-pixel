use std::marker::PhantomData;

use js::JsObject;

/// The classes of `./artboard.module.css`.
#[cfg_attr(rust_js, rust_js::css_module)]
pub struct Styles(PhantomData<JsObject>);

impl Styles {
    #[cfg_attr(rust_js, rust_js::link_name = "get artboard")]
    pub fn artboard(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get mockup")]
    pub fn mockup(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get preview-layer")]
    pub fn preview_layer(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get preview-text")]
    pub fn preview_text(&self) -> String {
        unreachable!()
    }
}

unsafe extern "Rust" {
    #[link_name = "./artboard.module.css#default"]
    pub safe static STYLES: &'static Styles;
}
