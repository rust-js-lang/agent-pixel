use std::marker::PhantomData;

use js::JsObject;

/// The classes of `./editor.module.css`.
#[cfg_attr(rust_js, rust_js::css_module)]
pub struct Styles(PhantomData<JsObject>);

impl Styles {
    #[cfg_attr(rust_js, rust_js::link_name = "get agent")]
    pub fn agent(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get caption")]
    pub fn caption(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get color")]
    pub fn color(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get component-library")]
    pub fn component_library(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get component-position")]
    pub fn component_position(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get editor")]
    pub fn editor(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get eyebrow")]
    pub fn eyebrow(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get imprint")]
    pub fn imprint(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get layer")]
    pub fn layer(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get layers")]
    pub fn layers(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get layout")]
    pub fn layout(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get legend")]
    pub fn legend(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get properties")]
    pub fn properties(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get server")]
    pub fn server(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get shape")]
    pub fn shape(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get shapes")]
    pub fn shapes(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get text-field")]
    pub fn text_field(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get text-layer")]
    pub fn text_layer(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get tools")]
    pub fn tools(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get workspace")]
    pub fn workspace(&self) -> String {
        unreachable!()
    }
}

unsafe extern "Rust" {
    #[link_name = "./editor.module.css#default"]
    pub safe static STYLES: &'static Styles;
}
