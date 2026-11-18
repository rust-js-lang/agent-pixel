use crate::projects::canopy_library;
use pixel_geometry::scene::{ComponentVariant, Scene};
use serde::Serialize;

#[derive(Serialize)]
pub struct Variant {
    pub id: &'static str,
    pub family: &'static str,
    pub name: &'static str,
}

const VARIANTS: [Variant; 12] = [
    Variant {
        id: "primary-button",
        family: "Button",
        name: "Primary",
    },
    Variant {
        id: "secondary-button",
        family: "Button",
        name: "Secondary",
    },
    Variant {
        id: "destructive-button",
        family: "Button",
        name: "Destructive",
    },
    Variant {
        id: "new-badge",
        family: "Status badge",
        name: "Neutral",
    },
    Variant {
        id: "active-badge",
        family: "Status badge",
        name: "Success",
    },
    Variant {
        id: "review-badge",
        family: "Status badge",
        name: "Warning",
    },
    Variant {
        id: "default-input",
        family: "Text input",
        name: "Default",
    },
    Variant {
        id: "focused-input",
        family: "Text input",
        name: "Focused",
    },
    Variant {
        id: "error-input",
        family: "Text input",
        name: "Error",
    },
    Variant {
        id: "off-switch",
        family: "Switch",
        name: "Off",
    },
    Variant {
        id: "on-switch",
        family: "Switch",
        name: "On",
    },
    Variant {
        id: "disabled-switch",
        family: "Switch",
        name: "Disabled",
    },
];

pub fn variants() -> &'static [Variant] {
    &VARIANTS
}

pub fn library_key() -> String {
    canopy_library().storage_key(false)
}

fn source() -> Result<Scene, String> {
    let project = canopy_library();
    Ok(project
        .load(false)?
        .and_then(|document| document.scene)
        .unwrap_or_else(|| project.initial_scene()))
}

fn variant_layers(scene: &Scene, variant: &Variant) -> ComponentVariant {
    let layers: Vec<_> = scene
        .layers
        .iter()
        .filter(|layer| {
            layer.id.strip_prefix(variant.id).is_some_and(|suffix| {
                suffix.is_empty() || (suffix.starts_with('-') && suffix != "-variant")
            })
        })
        .cloned()
        .collect();
    let assets = scene
        .assets
        .iter()
        .filter(|asset| {
            layers
                .iter()
                .any(|layer| layer.image_id.as_ref() == Some(&asset.id))
        })
        .cloned()
        .collect();
    ComponentVariant {
        id: variant.id.into(),
        family: variant.family.into(),
        layers,
        assets,
    }
}

pub fn component(id: &str) -> Result<ComponentVariant, String> {
    if let Some(library) = crate::EDITOR.with_borrow(|editor| editor.pinned_library.clone()) {
        return library
            .into_iter()
            .find(|variant| variant.id == id)
            .ok_or_else(|| "Unknown component variant in this project.".to_string());
    }
    let variant = variants()
        .iter()
        .find(|variant| variant.id == id)
        .ok_or("Unknown component variant.")?;
    Ok(variant_layers(&source()?, variant))
}

pub fn library() -> Result<Vec<ComponentVariant>, String> {
    match crate::EDITOR.with_borrow(|editor| editor.pinned_library.clone()) {
        Some(library) => Ok(library),
        None => live_library(),
    }
}

pub fn live_library() -> Result<Vec<ComponentVariant>, String> {
    let scene = source()?;
    Ok(variants()
        .iter()
        .map(|variant| variant_layers(&scene, variant))
        .collect())
}
