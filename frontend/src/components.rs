use crate::projects::canopy_library;
use pixel_geometry::scene::ComponentInsert;
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

pub fn placement(id: &str, x: i32, y: i32) -> Result<ComponentInsert, String> {
    let variant = variants()
        .iter()
        .find(|variant| variant.id == id)
        .ok_or("Unknown component variant.")?;
    let project = canopy_library();
    let scene = project
        .load(false)?
        .and_then(|document| document.scene)
        .unwrap_or_else(|| project.initial_scene());
    let layers = scene
        .layers
        .into_iter()
        .filter(|layer| {
            layer.id.strip_prefix(variant.id).is_some_and(|suffix| {
                suffix.is_empty() || (suffix.starts_with('-') && suffix != "-variant")
            })
        })
        .collect();
    Ok(ComponentInsert { layers, x, y })
}
