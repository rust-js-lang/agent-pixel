//! Frame geometry and clipping for the scene hierarchy.
use crate::{Dimensions, hierarchy::LayerNode, scene::Scene};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Frame {
    pub bounds: Rect,
    pub fill: String,
    pub clip: bool,
}

pub struct PaintNode {
    pub id: String,
    pub clip: Option<Rect>,
}

impl Rect {
    pub fn fits(self, board: Dimensions) -> bool {
        board.fits(self.x, self.y, self.width, self.height)
    }

    fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            width: ((self.x + self.width).min(other.x + other.width) - x).max(0),
            height: ((self.y + self.height).min(other.y + other.height) - y).max(0),
        }
    }
}

impl Frame {
    pub(crate) fn validate(&self, board: Dimensions) -> Result<(), &'static str> {
        if !self.bounds.fits(board) {
            return Err("Keep the frame inside the artboard.");
        }
        if !crate::scene::valid_fill(&self.fill) {
            return Err("Choose a six-digit hex frame fill.");
        }
        Ok(())
    }
}

impl Scene {
    pub(crate) fn set_frame(&mut self, id: &str, next: Frame) -> Result<(), &'static str> {
        next.validate(self.board)?;
        self.validate_hierarchy()?;
        let index = self
            .groups
            .iter()
            .position(|group| group.id == id)
            .ok_or("Unknown frame.")?;
        let previous = self.groups[index].frame.as_ref().ok_or("Unknown frame.")?;
        let dx = next.bounds.x - previous.bounds.x;
        let dy = next.bounds.y - previous.bounds.y;
        if dx != 0 || dy != 0 {
            let descendants = self.layer_tree();
            let root = find(&descendants, id).ok_or("Unknown frame.")?;
            let mut ids = std::collections::HashSet::new();
            collect(&root.children, &mut ids);
            for layer in &mut self.layers {
                if ids.contains(layer.id.as_str()) {
                    layer.x += dx;
                    layer.y += dy;
                }
            }
            for instance in &mut self.instances {
                if ids.contains(instance.id.as_str()) {
                    instance.x += dx;
                    instance.y += dy;
                }
            }
            for group in &mut self.groups {
                if ids.contains(group.id.as_str()) {
                    if let Some(frame) = &mut group.frame {
                        frame.bounds.x += dx;
                        frame.bounds.y += dy;
                    }
                }
            }
        }
        self.groups[index].frame = Some(next);
        Ok(())
    }

    pub fn paint_nodes(&self) -> Vec<PaintNode> {
        if !self.groups.iter().any(|group| group.frame.is_some()) {
            return self
                .layers
                .iter()
                .map(|layer| PaintNode {
                    id: layer.id.clone(),
                    clip: None,
                })
                .collect();
        }
        let frames: HashMap<_, _> = self
            .groups
            .iter()
            .filter_map(|group| group.frame.as_ref().map(|frame| (group.id.as_str(), frame)))
            .collect();
        let mut nodes = Vec::new();
        paint(&self.layer_tree(), &frames, None, &mut nodes);
        nodes
    }
}

fn find<'a>(nodes: &'a [LayerNode], id: &str) -> Option<&'a LayerNode> {
    for node in nodes {
        if node.id == id {
            return Some(node);
        }
        if let Some(found) = find(&node.children, id) {
            return Some(found);
        }
    }
    None
}

fn collect<'a>(nodes: &'a [LayerNode], ids: &mut std::collections::HashSet<&'a str>) {
    for node in nodes {
        ids.insert(node.id.as_str());
        collect(&node.children, ids);
    }
}

fn paint(
    tree: &[LayerNode],
    frames: &HashMap<&str, &Frame>,
    clip: Option<Rect>,
    nodes: &mut Vec<PaintNode>,
) {
    for node in tree.iter().rev() {
        let child_clip = if let Some(frame) = frames.get(node.id.as_str()) {
            nodes.push(PaintNode {
                id: node.id.clone(),
                clip,
            });
            if frame.clip {
                Some(clip.map_or(frame.bounds, |bounds| bounds.intersect(frame.bounds)))
            } else {
                clip
            }
        } else {
            if node.kind != "group" && node.kind != "instance" {
                nodes.push(PaintNode {
                    id: node.id.clone(),
                    clip,
                });
            }
            clip
        };
        paint(&node.children, frames, child_clip, nodes);
    }
}
