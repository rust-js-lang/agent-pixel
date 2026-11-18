use std::marker::PhantomData;

use js::JsObject;

/// The classes of `./dashboard.module.css`.
#[cfg_attr(rust_js, rust_js::css_module)]
pub struct Styles(PhantomData<JsObject>);

impl Styles {
    #[cfg_attr(rust_js, rust_js::link_name = "get create-project")]
    pub fn create_project(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get empty-preview")]
    pub fn empty_preview(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get orbit-preview")]
    pub fn orbit_preview(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get orbit-tile")]
    pub fn orbit_tile(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get preview-caption")]
    pub fn preview_caption(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-arrow")]
    pub fn project_arrow(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-card")]
    pub fn project_card(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-controls")]
    pub fn project_controls(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-dashboard")]
    pub fn project_dashboard(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-grid")]
    pub fn project_grid(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-heading")]
    pub fn project_heading(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-meta")]
    pub fn project_meta(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-preview")]
    pub fn project_preview(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-status")]
    pub fn project_status(&self) -> String {
        unreachable!()
    }

    #[cfg_attr(rust_js, rust_js::link_name = "get project-tile")]
    pub fn project_tile(&self) -> String {
        unreachable!()
    }
}

unsafe extern "Rust" {
    #[link_name = "./dashboard.module.css#default"]
    pub safe static STYLES: &'static Styles;
}
