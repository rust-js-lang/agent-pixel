use crate::{BOARD_HEIGHT, BOARD_WIDTH, fits_board};
use serde::{Deserialize, Serialize};

const VERSION: u8 = 1;
const MAX_LAYERS: usize = 1000;
const INK: &str = "#20231f";
const ORANGE: &str = "#ff5636";
const PAPER: &str = "#f5edda";

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Layer {
    pub id: String,
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub fill: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "fontSize", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Scene {
    pub version: u8,
    pub layers: Vec<Layer>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Document {
    pub revision: u64,
    pub scene: Option<Scene>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub id: Option<String>,
    pub kind: Option<String>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fill: Option<String>,
    pub text: Option<String>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<i32>,
}

impl Layer {
    fn orbit(id: &str, x: i32, y: i32, size: i32, fill: &str) -> Self {
        Self {
            id: id.into(),
            kind: "ellipse".into(),
            x,
            y,
            width: size,
            height: size,
            fill: fill.into(),
            text: None,
            font_size: None,
        }
    }

    fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.kind.as_str(), "ellipse" | "rectangle" | "text")
            || !self
                .fill
                .strip_prefix('#')
                .is_some_and(|hex| hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err("Choose a shape and a six-digit hex fill.");
        }
        if self.kind == "text" {
            if !self
                .text
                .as_ref()
                .is_some_and(|text| !text.trim().is_empty() && text.len() <= 512)
                || !self.font_size.is_some_and(|size| (8..=96).contains(&size))
            {
                return Err("Text needs content and a font size from 8 to 96.");
            }
        } else if self.text.is_some() || self.font_size.is_some() {
            return Err("Text properties require a text layer.");
        }
        if !fits_board(self.x, self.y, self.width, self.height) {
            return Err("Keep the entire shape inside the artboard.");
        }
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("Invalid layer ID.");
        }
        Ok(())
    }
}

impl Scene {
    pub fn blank() -> Self {
        Self {
            version: VERSION,
            layers: Vec::new(),
        }
    }

    pub fn poster() -> Self {
        Self {
            version: VERSION,
            layers: vec![
                Layer::orbit("outer-disc", 44, 68, 392, INK),
                Layer::orbit("orange-disc", 64, 88, 352, ORANGE),
                Layer::orbit("middle-disc", 84, 108, 312, INK),
                Layer::orbit("inner-disc", 104, 128, 272, ORANGE),
                Layer::orbit("core", 124, 148, 232, INK),
                Layer::orbit("moon", 150, 175, 178, PAPER),
                Layer::orbit("satellite", 305, 312, 74, ORANGE),
            ],
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != VERSION || self.layers.len() > MAX_LAYERS {
            return Err("Unsupported document.");
        }
        let mut ids = std::collections::HashSet::new();
        for layer in &self.layers {
            layer.validate()?;
            if !ids.insert(&layer.id) {
                return Err("Duplicate layer ID.");
            }
        }
        Ok(())
    }

    pub fn edit(&mut self, patch: Patch, new_id: String) -> Result<String, &'static str> {
        let Patch {
            id,
            kind,
            x,
            y,
            width,
            height,
            fill,
            text,
            font_size,
        } = patch;
        if kind.is_none()
            && x.is_none()
            && y.is_none()
            && width.is_none()
            && height.is_none()
            && fill.is_none()
            && text.is_none()
            && font_size.is_none()
        {
            return Err("Specify shape properties.");
        }
        let index = match id {
            Some(id) => Some(
                self.layers
                    .iter()
                    .position(|layer| layer.id == id)
                    .ok_or("Unknown layer.")?,
            ),
            None if self.layers.len() >= MAX_LAYERS => return Err("Layer limit reached."),
            None => None,
        };
        let mut layer = match index {
            Some(index) => self.layers[index].clone(),
            None => Layer {
                id: new_id,
                kind: "rectangle".into(),
                x: 40,
                y: 40,
                width: 80,
                height: 80,
                fill: ORANGE.into(),
                text: None,
                font_size: None,
            },
        };
        if let Some(kind) = kind {
            layer.kind = kind;
        }
        if let Some(x) = x {
            layer.x = x;
        }
        if let Some(y) = y {
            layer.y = y;
        }
        if let Some(width) = width {
            layer.width = width;
        }
        if let Some(height) = height {
            layer.height = height;
        }
        if let Some(fill) = fill {
            layer.fill = fill;
        }
        if let Some(text) = text {
            layer.text = Some(text);
        }
        if let Some(font_size) = font_size {
            layer.font_size = Some(font_size);
        }
        layer.validate()?;
        let id = layer.id.clone();
        match index {
            Some(index) => self.layers[index] = layer,
            None => self.layers.push(layer),
        }
        Ok(id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Design {
    pub board: Dimensions,
    pub scene: Scene,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Dimensions {
    pub width: i32,
    pub height: i32,
}

pub fn worker_command(
    scene: &mut Scene,
    command: &str,
    input: &str,
    id: String,
) -> Result<String, String> {
    let edited = match command {
        "read" => None,
        "restore" => {
            let restored: Scene =
                serde_json::from_str(input).map_err(|_| "Unsupported document.")?;
            restored.validate()?;
            *scene = restored;
            None
        }
        "edit" => {
            let patch: Patch =
                serde_json::from_str(input).map_err(|_| "Specify valid shape properties.")?;
            Some(scene.edit(patch, id)?)
        }
        _ => return Err("Unknown command.".into()),
    };
    let design = Design {
        scene: scene.clone(),
        board: Dimensions {
            width: BOARD_WIDTH,
            height: BOARD_HEIGHT,
        },
        edited,
    };
    serde_json::to_string(&design).map_err(|error| error.to_string())
}
