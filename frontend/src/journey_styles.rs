use std::marker::PhantomData;

use js::JsObject;

/// The classes of `./journey.module.css`.
#[cfg_attr(rust_js, rust_js::css_module)]
pub struct Styles(PhantomData<JsObject>);

impl Styles {
    #[cfg_attr(rust_js, rust_js::link_name = "get editor-navigation")]
    pub fn editor_navigation(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get journey-actions")]
    pub fn journey_actions(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get journey-mode")]
    pub fn journey_mode(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get journey-navigation")]
    pub fn journey_navigation(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get journey-preview")]
    pub fn journey_preview(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get journey-steps")]
    pub fn journey_steps(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get prototype-canvas")]
    pub fn prototype_canvas(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get prototype-guide")]
    pub fn prototype_guide(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get prototype-hotspot")]
    pub fn prototype_hotspot(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get simulate-failure")]
    pub fn simulate_failure(&self) -> String {
        unreachable!()
    }
}

unsafe extern "Rust" {
    #[link_name = "./journey.module.css#default"]
    pub safe static STYLES: &'static Styles;
}
