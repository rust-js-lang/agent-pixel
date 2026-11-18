//! Parent relationships and stacking order for layer groups and linked instances.
use crate::scene::Scene;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const MAX_DEPTH: usize = 16;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct LayerGroup {
    pub id: String,
    pub name: String,
    pub children: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerNode {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub children: Vec<LayerNode>,
}

impl Scene {
    pub(crate) fn validate_hierarchy(&self) -> Result<(), &'static str> {
        let mut ids = HashSet::new();
        for layer in &self.layers {
            ids.insert(layer.id.as_str());
        }
        for instance in &self.instances {
            if !crate::scene::valid_id(&instance.id) || !ids.insert(instance.id.as_str()) {
                return Err("Invalid or duplicate instance ID.");
            }
        }
        for group in &self.groups {
            if !crate::scene::valid_id(&group.id)
                || !crate::bundle::valid_title(&group.name)
                || !ids.insert(group.id.as_str())
            {
                return Err("Invalid layer group.");
            }
        }
        let mut parents = HashMap::new();
        for instance in &self.instances {
            for id in &instance.layer_ids {
                parents.insert(id.as_str(), instance.id.as_str());
            }
        }
        for group in &self.groups {
            for id in &group.children {
                if !ids.contains(id.as_str())
                    || parents.insert(id.as_str(), group.id.as_str()).is_some()
                {
                    return Err("Each group child must exist and have one parent.");
                }
            }
        }
        for id in ids {
            let mut current = id;
            for depth in 0..=MAX_DEPTH {
                match parents.get(current) {
                    None => break,
                    Some(parent) if **parent == *id || depth == MAX_DEPTH => {
                        return Err("Groups cannot contain cycles or exceed 16 levels.");
                    }
                    Some(parent) => current = parent,
                }
            }
        }
        Ok(())
    }

    pub fn parent_group(&self, id: &str) -> Option<&str> {
        self.groups
            .iter()
            .find(|group| group.children.iter().any(|child| child == id))
            .map(|group| group.id.as_str())
    }

    pub fn is_valid_parent(&self, id: &str, candidate: &str) -> bool {
        let mut parent = Some(candidate);
        while let Some(current) = parent {
            if current == id {
                return false;
            }
            parent = self.parent_group(current);
        }
        true
    }

    pub(crate) fn detach_from_group(&mut self, id: &str) {
        for group in &mut self.groups {
            group.children.retain(|child| child != id);
        }
    }

    pub(crate) fn create_group(
        &mut self,
        id: String,
        name: String,
        parent_id: Option<String>,
    ) -> Result<String, &'static str> {
        if !crate::bundle::valid_title(&name) {
            return Err("Enter a group name between 1 and 80 characters.");
        }
        self.groups.push(LayerGroup {
            id: id.clone(),
            name,
            children: Vec::new(),
        });
        self.move_layer_node(&id, parent_id.as_deref())?;
        Ok(id)
    }

    pub(crate) fn move_layer_node(
        &mut self,
        id: &str,
        parent_id: Option<&str>,
    ) -> Result<(), &'static str> {
        if self.instance_for_layer(id).is_some() {
            return Err("Move the whole linked instance to change its parent.");
        }
        if !self.layers.iter().any(|layer| layer.id == id)
            && !self.instances.iter().any(|instance| instance.id == id)
            && !self.groups.iter().any(|group| group.id == id)
        {
            return Err("Unknown layer or group.");
        }
        self.detach_from_group(id);
        if let Some(parent_id) = parent_id {
            let parent = self
                .groups
                .iter_mut()
                .find(|group| group.id == parent_id)
                .ok_or("Unknown parent group.")?;
            parent.children.push(id.to_owned());
        }
        Ok(())
    }

    pub(crate) fn rename_group(&mut self, id: &str, name: String) -> Result<(), &'static str> {
        if !crate::bundle::valid_title(&name) {
            return Err("Enter a group name between 1 and 80 characters.");
        }
        let group = self
            .groups
            .iter_mut()
            .find(|group| group.id == id)
            .ok_or("Unknown group.")?;
        group.name = name;
        Ok(())
    }

    pub(crate) fn ungroup(&mut self, id: &str) -> Result<(), &'static str> {
        let index = self
            .groups
            .iter()
            .position(|group| group.id == id)
            .ok_or("Unknown group.")?;
        let group = self.groups.remove(index);
        if let Some(parent) = self
            .groups
            .iter_mut()
            .find(|parent| parent.children.iter().any(|child| child == id))
        {
            parent.children.retain(|child| child != id);
            parent.children.extend(group.children);
        }
        Ok(())
    }

    pub fn layer_tree(&self) -> Vec<LayerNode> {
        let mut tree = HashMap::new();
        let mut parents = HashMap::new();
        let mut rank = HashMap::new();
        let mut nodes = HashMap::new();
        for group in &self.groups {
            nodes.insert(group.id.as_str(), (group.name.clone(), "group"));
            for child in &group.children {
                parents.insert(child.as_str(), group.id.as_str());
            }
        }
        for instance in &self.instances {
            nodes.insert(instance.id.as_str(), (instance.family.clone(), "instance"));
            for child in &instance.layer_ids {
                parents.insert(child.as_str(), instance.id.as_str());
            }
        }
        for (index, layer) in self.layers.iter().enumerate() {
            nodes.insert(
                layer.id.as_str(),
                (self.layer_name(layer), layer.kind.as_str()),
            );
            let mut id = layer.id.as_str();
            for _ in 0..=MAX_DEPTH {
                rank.insert(id, index);
                let Some(parent) = parents.get(id) else { break };
                id = parent;
            }
        }
        for (index, group) in self.groups.iter().enumerate() {
            rank.entry(group.id.as_str())
                .or_insert(self.layers.len() + index);
        }
        for &id in nodes.keys() {
            let parent = parents.get(id).copied();
            tree.entry(parent).or_insert_with(Vec::new).push(id);
        }
        for children in tree.values_mut() {
            children.sort_by_key(|id| rank.get(id).copied().unwrap_or_default());
        }
        rows(None, &tree, &nodes)
    }

    fn layer_name(&self, layer: &crate::scene::Layer) -> String {
        self.image_for_layer(layer).map_or_else(|| layer.name(), |asset| asset.image.name.clone())
    }

    pub(crate) fn normalize_layer_order(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        let mut order = HashMap::new();
        let mut index = 0;
        paint_order(&self.layer_tree(), &mut order, &mut index);
        self.layers
            .sort_by_key(|layer| order.get(&layer.id).copied().unwrap_or_default());
    }
}

fn rows(
    parent: Option<&str>,
    tree: &HashMap<Option<&str>, Vec<&str>>,
    nodes: &HashMap<&str, (String, &str)>,
) -> Vec<LayerNode> {
    let Some(children) = tree.get(&parent) else {
        return Vec::new();
    };
    children
        .iter()
        .rev()
        .map(|&id| {
            let (name, kind) = nodes.get(id).expect("Tree node");
            LayerNode {
                id: id.to_owned(),
                name: name.clone(),
                kind: (*kind).to_owned(),
                children: rows(Some(id), tree, nodes),
            }
        })
        .collect()
}

fn paint_order(rows: &[LayerNode], order: &mut HashMap<String, usize>, index: &mut usize) {
    for row in rows.iter().rev() {
        if row.kind != "group" && row.kind != "instance" {
            order.insert(row.id.clone(), *index);
            *index += 1;
        } else {
            paint_order(&row.children, order, index);
        }
    }
}
