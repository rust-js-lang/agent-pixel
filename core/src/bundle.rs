//! Portable projects and validation shared across native Rust and rust-js.
use crate::scene::{ComponentVariant, Scene};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const VERSION: u8 = 1;
pub const MAX_BYTES: usize = 2_000_000;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenInfo {
    pub id: String,
    pub title: String,
    pub action: Option<Action>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub layer_id: String,
    pub label: String,
    pub next: String,
    pub failure: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct BundleScreen {
    pub info: ScreenInfo,
    pub scene: Scene,
}

#[derive(Deserialize, Serialize)]
pub struct ProjectBundle {
    pub version: u8,
    pub title: String,
    pub poster: bool,
    pub handoff: Option<String>,
    pub screens: Vec<BundleScreen>,
    pub library: Vec<ComponentVariant>,
}

pub fn valid_title(title: &str) -> bool {
    !title.trim().is_empty() && title.chars().count() <= 80
}

impl ProjectBundle {
    pub fn parse(json: &str) -> Result<Self, &'static str> {
        if json.len() > MAX_BYTES {
            return Err("Project bundle exceeds 2 MB.");
        }
        let bundle: Self =
            serde_json::from_str(json).map_err(|_| "Invalid project bundle JSON.")?;
        bundle.validate()?;
        Ok(bundle)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != VERSION || !valid_title(&self.title) || self.screens.is_empty() {
            return Err("Unsupported project bundle.");
        }
        let mut ids = HashSet::new();
        for screen in &self.screens {
            let id = &screen.info.id;
            if id.is_empty()
                || id.len() > 64
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || !valid_title(&screen.info.title)
                || !ids.insert(id)
            {
                return Err("Invalid or duplicate screen.");
            }
            screen.scene.validate()?;
        }
        for screen in &self.screens {
            if let Some(action) = &screen.info.action {
                if action.label.trim().is_empty()
                    || !ids.contains(&action.next)
                    || action.failure.as_ref().is_some_and(|id| !ids.contains(id))
                    || !screen
                        .scene
                        .layers
                        .iter()
                        .any(|layer| layer.id == action.layer_id)
                {
                    return Err("Invalid prototype action.");
                }
            }
        }
        let mut variant_ids = HashSet::new();
        for variant in &self.library {
            if variant.id.is_empty()
                || variant.family.is_empty()
                || variant.layers.is_empty()
                || !variant_ids.insert(&variant.id)
            {
                return Err("Invalid or duplicate component variant.");
            }
            let mut source = Scene::blank();
            source.board = crate::Dimensions::LARGEST;
            source.layers = variant.layers.clone();
            source.assets = variant.assets.clone();
            source.validate()?;
        }
        for screen in &self.screens {
            for instance in &screen.scene.instances {
                if !self.library.iter().any(|variant| {
                    variant.id == instance.variant && variant.family == instance.family
                }) {
                    return Err("Project bundle is missing a linked component.");
                }
            }
        }
        if self
            .handoff
            .as_ref()
            .is_some_and(|url| !url.starts_with("https://"))
        {
            return Err("Invalid handoff URL.");
        }
        Ok(())
    }
}
