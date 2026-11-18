use crate::{
    BOARD_HEIGHT, BOARD_WIDTH,
    assets::{ImageAsset, ImagePlacement, MAX_SCENE_IMAGE_BYTES},
    fits_board,
    hierarchy::LayerGroup,
};
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
    #[serde(rename = "imageId", skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Scene {
    pub version: u8,
    pub layers: Vec<Layer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<ComponentInstance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<ImageAsset>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<LayerGroup>,
}

#[derive(Clone, Default, PartialEq, Deserialize, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ComponentInstance {
    pub id: String,
    pub variant: String,
    pub family: String,
    pub x: i32,
    pub y: i32,
    #[serde(rename = "layerIds")]
    pub layer_ids: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ComponentVariant {
    pub id: String,
    pub family: String,
    pub layers: Vec<Layer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<ImageAsset>,
}

#[derive(Deserialize, Serialize)]
pub struct ComponentInsert {
    pub variant: ComponentVariant,
    pub x: i32,
    pub y: i32,
}

#[derive(Deserialize, Serialize)]
pub struct ComponentSwitch {
    pub id: String,
    pub variant: ComponentVariant,
}

fn placed_layers(
    variant: &ComponentVariant,
    id: &str,
    x: i32,
    y: i32,
    fit: bool,
) -> Result<Vec<Layer>, &'static str> {
    if variant.layers.is_empty() {
        return Err("Cannot insert an empty component.");
    }
    let (mut left, mut top, mut right, mut bottom) = (BOARD_WIDTH, BOARD_HEIGHT, 0, 0);
    for layer in &variant.layers {
        layer.validate()?;
        left = left.min(layer.x);
        top = top.min(layer.y);
        right = right.max(layer.x + layer.width);
        bottom = bottom.max(layer.y + layer.height);
    }
    let (width, height) = (right - left, bottom - top);
    let (x, y) = if fit {
        (
            x.clamp(0, BOARD_WIDTH - width),
            y.clamp(0, BOARD_HEIGHT - height),
        )
    } else {
        (x, y)
    };
    if !fits_board(x, y, width, height) {
        return Err("Keep the entire component inside the artboard.");
    }
    Ok(variant
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let mut layer = layer.clone();
            layer.id = format!("{id}-{index}");
            layer.x = x + (layer.x - left);
            layer.y = y + (layer.y - top);
            layer
        })
        .collect())
}

pub(crate) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl Layer {
    pub fn name(&self) -> String {
        if let Some(text) = &self.text {
            text.clone()
        } else if self.id.len() == 36 || self.id.as_bytes().get(36) == Some(&b'-') {
            let prefix: String = self.id.chars().take(4).collect();
            format!("{} {}", self.kind, prefix)
        } else {
            self.id.replace('-', " ")
        }
    }

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
            image_id: None,
        }
    }

    fn validate(&self) -> Result<(), &'static str> {
        if !matches!(
            self.kind.as_str(),
            "ellipse" | "rectangle" | "text" | "image"
        ) || !self
            .fill
            .strip_prefix('#')
            .is_some_and(|hex| hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err("Choose a shape and a six-digit hex fill.");
        }
        if (self.kind == "image") != self.image_id.is_some() {
            return Err("Image layers need an asset reference.");
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
        if !valid_id(&self.id) {
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
            instances: Vec::new(),
            assets: Vec::new(),
            groups: Vec::new(),
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
            instances: Vec::new(),
            assets: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != VERSION || self.layers.len() + self.groups.len() > MAX_LAYERS {
            return Err("Unsupported document.");
        }
        let mut assets = std::collections::HashSet::new();
        let mut bytes = 0;
        for asset in &self.assets {
            if !valid_id(&asset.id) || !assets.insert(&asset.id) {
                return Err("Invalid or duplicate asset ID.");
            }
            asset.image.validate()?;
            bytes += asset.image.data_url.len();
            if bytes > MAX_SCENE_IMAGE_BYTES {
                return Err("Image assets exceed the artboard storage limit.");
            }
        }
        let mut ids = std::collections::HashSet::new();
        for layer in &self.layers {
            layer.validate()?;
            if layer
                .image_id
                .as_ref()
                .is_some_and(|id| !assets.contains(id))
            {
                return Err("Image layer refers to a missing asset.");
            }
            if !ids.insert(&layer.id) {
                return Err("Duplicate layer ID.");
            }
        }
        let mut instances = std::collections::HashSet::new();
        let mut linked_layers = std::collections::HashSet::new();
        for instance in &self.instances {
            if !instances.insert(&instance.id)
                || instance.layer_ids.is_empty()
                || !instance
                    .layer_ids
                    .iter()
                    .all(|id| ids.contains(id) && linked_layers.insert(id))
            {
                return Err("Invalid component instance.");
            }
        }
        self.validate_hierarchy()
    }

    pub fn image_for_layer(&self, layer: &Layer) -> Option<&ImageAsset> {
        let id = layer.image_id.as_ref()?;
        self.assets.iter().find(|asset| &asset.id == id)
    }

    fn prune_assets(&mut self) {
        self.assets.retain(|asset| {
            self.layers
                .iter()
                .any(|layer| layer.image_id.as_ref() == Some(&asset.id))
        });
    }

    fn merge_assets(&mut self, assets: &[ImageAsset]) -> Result<(), &'static str> {
        for asset in assets {
            if let Some(existing) = self.assets.iter().find(|existing| existing.id == asset.id) {
                if existing != asset {
                    return Err("An image asset ID already refers to different data.");
                }
            } else {
                self.assets.push(asset.clone());
            }
        }
        Ok(())
    }

    fn place_image(
        &mut self,
        placement: ImagePlacement,
        id: String,
    ) -> Result<String, &'static str> {
        if self.layers.len() + self.groups.len() >= MAX_LAYERS {
            return Err("Layer limit reached.");
        }
        placement.image.validate()?;
        let existing = self
            .assets
            .iter()
            .find(|asset| asset.image == placement.image);
        let layer = Layer {
            id: id.clone(),
            kind: "image".into(),
            x: placement.x,
            y: placement.y,
            width: placement.width,
            height: placement.height,
            fill: "#ffffff".into(),
            text: None,
            font_size: None,
            image_id: Some(existing.map_or_else(|| id.clone(), |asset| asset.id.clone())),
        };
        layer.validate()?;
        if existing.is_none() {
            let bytes = self
                .assets
                .iter()
                .map(|asset| asset.image.data_url.len())
                .sum::<usize>();
            if bytes + placement.image.data_url.len() > MAX_SCENE_IMAGE_BYTES {
                return Err("Image assets exceed the artboard storage limit.");
            }
            self.assets.push(ImageAsset {
                id: id.clone(),
                image: placement.image,
            });
        }
        self.layers.push(layer);
        Ok(id)
    }

    pub fn instance_for_layer(&self, id: &str) -> Option<&ComponentInstance> {
        self.instances
            .iter()
            .find(|instance| instance.layer_ids.iter().any(|layer| layer == id))
    }

    fn insert_component(
        &mut self,
        insert: ComponentInsert,
        id: String,
    ) -> Result<Vec<String>, &'static str> {
        let layers = placed_layers(&insert.variant, &id, insert.x, insert.y, false)?;
        let ids: Vec<_> = layers.iter().map(|layer| layer.id.clone()).collect();
        let mut candidate = self.clone();
        candidate.layers.extend(layers);
        candidate.merge_assets(&insert.variant.assets)?;
        candidate.instances.push(ComponentInstance {
            id,
            variant: insert.variant.id,
            family: insert.variant.family,
            x: insert.x,
            y: insert.y,
            layer_ids: ids.clone(),
        });
        candidate.validate()?;
        *self = candidate;
        Ok(ids)
    }

    fn replace_instance(
        &mut self,
        index: usize,
        variant: &ComponentVariant,
        fit: bool,
    ) -> Result<String, &'static str> {
        let instance = &mut self.instances[index];
        if instance.family != variant.family {
            return Err("Choose a variant from the same component family.");
        }
        let layers = placed_layers(variant, &instance.id, instance.x, instance.y, fit)?;
        let position = self
            .layers
            .iter()
            .position(|layer| instance.layer_ids.contains(&layer.id))
            .ok_or("Missing instance layers.")?;
        self.layers
            .retain(|layer| !instance.layer_ids.contains(&layer.id));
        instance.variant = variant.id.clone();
        instance.x = layers
            .iter()
            .map(|layer| layer.x)
            .min()
            .expect("Component layers");
        instance.y = layers
            .iter()
            .map(|layer| layer.y)
            .min()
            .expect("Component layers");
        instance.layer_ids = layers.iter().map(|layer| layer.id.clone()).collect();
        let selected = instance.layer_ids[0].clone();
        self.layers.splice(position..position, layers);
        self.merge_assets(&variant.assets)?;
        self.prune_assets();
        Ok(selected)
    }

    fn switch_component(&mut self, switch: ComponentSwitch) -> Result<String, &'static str> {
        let index = self
            .instances
            .iter()
            .position(|instance| instance.id == switch.id)
            .ok_or("Unknown component instance.")?;
        let mut candidate = self.clone();
        let selected = candidate.replace_instance(index, &switch.variant, false)?;
        candidate.validate()?;
        *self = candidate;
        Ok(selected)
    }

    pub fn refresh_components(
        &mut self,
        variants: &[ComponentVariant],
    ) -> Result<(), &'static str> {
        let mut candidate = self.clone();
        for index in 0..candidate.instances.len() {
            if let Some(variant) = variants
                .iter()
                .find(|variant| variant.id == candidate.instances[index].variant)
            {
                candidate.replace_instance(index, variant, true)?;
            }
        }
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    fn delete_layer(&mut self, id: &str) -> Result<(), &'static str> {
        if let Some(index) = self.instances.iter().position(|instance| {
            instance.id == id || instance.layer_ids.iter().any(|layer| layer == id)
        }) {
            let instance = self.instances.remove(index);
            self.detach_from_group(&instance.id);
            self.layers
                .retain(|layer| !instance.layer_ids.contains(&layer.id));
        } else {
            let index = self
                .layers
                .iter()
                .position(|layer| layer.id == id)
                .ok_or("Unknown layer.")?;
            self.layers.remove(index);
            self.detach_from_group(id);
        }
        self.prune_assets();
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
        if id
            .as_deref()
            .is_some_and(|id| self.instance_for_layer(id).is_some())
        {
            return Err("Edit the main component in Canopy UI or switch the instance variant.");
        }
        let index = match id {
            Some(id) => Some(
                self.layers
                    .iter()
                    .position(|layer| layer.id == id)
                    .ok_or("Unknown layer.")?,
            ),
            None if self.layers.len() + self.groups.len() >= MAX_LAYERS => {
                return Err("Layer limit reached.");
            }
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
                image_id: None,
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

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Author {
    #[default]
    Person,
    Agent,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub label: String,
    pub author: Author,
}

#[derive(Default, Serialize, Deserialize)]
pub struct History {
    pub undo: Vec<HistoryEntry>,
    pub redo: Vec<HistoryEntry>,
}

struct Transaction {
    scene: Scene,
    entry: HistoryEntry,
}

pub struct Session {
    scene: Scene,
    revision: String,
    undo: Vec<Transaction>,
    redo: Vec<Transaction>,
    library: Vec<ComponentVariant>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    pub revision: String,
    pub edits: Vec<Edit>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    Patch {
        layer: Patch,
    },
    Delete {
        id: String,
    },
    PlaceImage {
        placement: ImagePlacement,
    },
    CreateGroup {
        name: String,
        #[serde(rename = "parentId")]
        parent_id: Option<String>,
    },
    SetLayerParent {
        id: String,
        #[serde(rename = "parentId")]
        parent_id: Option<String>,
    },
    RenameGroup {
        id: String,
        name: String,
    },
    Ungroup {
        id: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Design {
    pub board: Dimensions,
    pub scene: Scene,
    pub revision: String,
    pub history: History,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Dimensions {
    pub width: i32,
    pub height: i32,
}

const HISTORY_LIMIT: usize = 32;
const BATCH_LIMIT: usize = 100;

impl Batch {
    pub fn validate_size(&self) -> Result<(), &'static str> {
        if self.edits.is_empty() || self.edits.len() > BATCH_LIMIT {
            return Err("Submit between 1 and 100 edits.");
        }
        Ok(())
    }
}

impl Session {
    pub fn new(scene: Scene) -> Self {
        Self {
            scene,
            revision: String::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            library: Vec::new(),
        }
    }

    fn commit(&mut self, candidate: Scene, revision: &str, label: &str, author: Author) {
        if candidate == self.scene {
            return;
        }
        let previous = std::mem::replace(&mut self.scene, candidate);
        self.undo.push(Transaction {
            scene: previous,
            entry: HistoryEntry {
                label: label.into(),
                author,
            },
        });
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.revision = revision.into();
    }

    fn travel(&mut self, undo: bool, revision: &str) -> Result<(), &'static str> {
        let (from, to) = if undo {
            (&mut self.undo, &mut self.redo)
        } else {
            (&mut self.redo, &mut self.undo)
        };
        let transaction = from.last().ok_or("No edit in that direction.")?;
        let mut candidate = transaction.scene.clone();
        if !candidate.instances.is_empty() {
            candidate.refresh_components(&self.library)?;
        }
        let previous = std::mem::replace(&mut self.scene, candidate);
        let transaction = from.pop().expect("History entry");
        to.push(Transaction {
            scene: previous,
            entry: transaction.entry,
        });
        self.revision = revision.into();
        Ok(())
    }

    fn apply_batch(
        &mut self,
        batch: Batch,
        id: &str,
        author: Author,
    ) -> Result<Option<String>, &'static str> {
        if batch.revision != self.revision {
            return Err("Design changed. Read its latest revision before applying edits.");
        }
        batch.validate_size()?;
        let mut candidate = self.scene.clone();
        let mut selected = None;
        for (index, edit) in batch.edits.into_iter().enumerate() {
            selected = match edit {
                Edit::Patch { layer } => Some(candidate.edit(layer, format!("{id}-{index}"))?),
                Edit::PlaceImage { placement } => {
                    Some(candidate.place_image(placement, format!("{id}-{index}"))?)
                }
                Edit::CreateGroup { name, parent_id } => {
                    Some(candidate.create_group(format!("{id}-{index}"), name, parent_id)?)
                }
                Edit::SetLayerParent { id, parent_id } => {
                    candidate.move_layer_node(&id, parent_id.as_deref())?;
                    Some(id)
                }
                Edit::RenameGroup { id, name } => {
                    candidate.rename_group(&id, name)?;
                    Some(id)
                }
                Edit::Ungroup { id } => {
                    candidate.ungroup(&id)?;
                    None
                }
                Edit::Delete { id } => {
                    candidate.delete_layer(&id)?;
                    None
                }
            };
        }
        if !candidate.groups.is_empty() || !self.scene.groups.is_empty() {
            candidate.validate()?;
            candidate.normalize_layer_order();
        }
        self.commit(candidate, id, "Edit layers", author);
        Ok(selected)
    }

    pub fn command(
        &mut self,
        command: &str,
        input: &str,
        id: String,
        author: Author,
    ) -> Result<String, String> {
        let mut inserted = None;
        let edited = match command {
            "read" => None,
            "restore" => {
                let mut restored: Scene =
                    serde_json::from_str(input).map_err(|_| "Unsupported document.")?;
                restored.validate()?;
                restored.normalize_layer_order();
                *self = Self::new(restored);
                self.revision = id;
                None
            }
            "undo" | "redo" => {
                let revision: String = serde_json::from_str(input)
                    .map_err(|_| "Provide the current design revision.")?;
                if revision != self.revision {
                    return Err(
                        "Design changed. Read its latest revision before using history.".into(),
                    );
                }
                self.travel(command == "undo", &id)?;
                None
            }
            "batch" => {
                let batch = serde_json::from_str(input)
                    .map_err(|_| "Specify valid edits and a revision.")?;
                self.apply_batch(batch, &id, author)?
            }
            "edit" => {
                let patch: Patch =
                    serde_json::from_str(input).map_err(|_| "Specify valid shape properties.")?;
                let mut candidate = self.scene.clone();
                let selected = candidate.edit(patch, id.clone())?;
                self.commit(candidate, &id, "Edit layer", author);
                Some(selected)
            }
            "insert" => {
                let component =
                    serde_json::from_str(input).map_err(|_| "Specify a valid component.")?;
                let mut candidate = self.scene.clone();
                let ids = candidate.insert_component(component, id.clone())?;
                let selected = ids.first().cloned();
                self.commit(candidate, &id, "Insert component", author);
                inserted = Some(ids);
                selected
            }
            "switch" => {
                let switch = serde_json::from_str(input)
                    .map_err(|_| "Specify a valid component variant.")?;
                let mut candidate = self.scene.clone();
                let selected = candidate.switch_component(switch)?;
                self.commit(candidate, &id, "Switch component variant", author);
                Some(selected)
            }
            "refresh_components" => {
                let variants: Vec<ComponentVariant> =
                    serde_json::from_str(input).map_err(|_| "Invalid component library.")?;
                let mut candidate = self.scene.clone();
                candidate.refresh_components(&variants)?;
                if candidate != self.scene {
                    self.scene = candidate;
                    self.revision = id;
                }
                self.library = variants;
                None
            }
            _ => return Err("Unknown command.".into()),
        };
        let design = Design {
            scene: self.scene.clone(),
            board: Dimensions {
                width: BOARD_WIDTH,
                height: BOARD_HEIGHT,
            },
            revision: self.revision.clone(),
            history: History {
                undo: self.undo.iter().map(|entry| entry.entry.clone()).collect(),
                redo: self
                    .redo
                    .iter()
                    .rev()
                    .map(|entry| entry.entry.clone())
                    .collect(),
            },
            edited,
            inserted,
        };
        serde_json::to_string(&design).map_err(|error| error.to_string())
    }
}
