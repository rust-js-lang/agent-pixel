use std::marker::PhantomData;

use js::JsObject;

/// The classes of `./shell.module.css`.
#[cfg_attr(rust_js, rust_js::css_module)]
pub struct Styles(PhantomData<JsObject>);

impl Styles {
    #[cfg_attr(rust_js, rust_js::link_name = "get brand")]
    pub fn brand(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get crumb")]
    pub fn crumb(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get footer")]
    pub fn footer(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get header")]
    pub fn header(&self) -> String {
        unreachable!()
    }
}

unsafe extern "Rust" {
    #[link_name = "./shell.module.css#default"]
    pub safe static STYLES: &'static Styles;
}
