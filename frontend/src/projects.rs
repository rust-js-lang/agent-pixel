use std::cell::RefCell;

use crate::{browser, storage};
use pixel_geometry::bundle::ProjectBundle;
use pixel_geometry::scene::{Document, Scene};
use serde::{Deserialize, Serialize};
use webapi::global::crypto;
use webapi::{Event, EventTargetExt, events};

const ORBIT: &str = "orbit";
const CURRENT: &str = "current";
const CANOPY: &str = "canopy-ui";
const WEB_APP: &str = "web-app";
const LEGACY_CATALOG: &str = "agent-pixel:projects:v1";
const MIGRATED: &str = "legacy-catalog-migrated";
pub const CHANGED: &str = "agent-pixel:projects-changed";

#[derive(Deserialize, Serialize)]
struct SavedProject {
    project: Project,
    bundle: Option<String>,
    order: i64,
}

thread_local! {
    static CATALOG: RefCell<Vec<SavedProject>> = const { RefCell::new(Vec::new()) };
    static WARNING: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub title: String,
}

impl Project {
    pub fn is_poster(&self) -> bool {
        self.id == ORBIT
    }

    pub fn href(&self) -> String {
        format!("?project={}", self.id)
    }

    pub fn storage_key(&self, server: bool) -> String {
        match self.id.as_str() {
            ORBIT if server => "agent-pixel:poster:v1:server".into(),
            ORBIT => "agent-pixel:poster:v1".into(),
            WEB_APP => "agent-pixel:web-app:v1".into(),
            id => format!("agent-pixel:project:{id}:v1"),
        }
    }

    fn legacy_bundle_key(&self) -> String {
        format!("agent-pixel:bundle:{}", self.id)
    }

    pub fn screen_key(&self, server: bool, legacy_base_id: &str, screen_id: &str) -> String {
        let base = self.storage_key(server);
        if screen_id == legacy_base_id {
            base
        } else {
            format!("{base}:{screen_id}")
        }
    }

    pub fn initial_scene(&self) -> Scene {
        match self.id.as_str() {
            ORBIT => Scene::poster(),
            CURRENT => serde_json::from_str(include_str!("../demos/current.json"))
                .expect("Valid Current demo document"),
            CANOPY => serde_json::from_str(include_str!("../demos/canopy-ui.json"))
                .expect("Valid Canopy UI demo document"),
            crate::journey::PROJECT => crate::journey::SCREENS[0].initial_scene(),
            _ => Scene::blank(),
        }
    }

    pub fn load(&self, server: bool) -> Result<Option<Document>, String> {
        load_document(&self.storage_key(server))
    }
}

pub fn load_document(key: &str) -> Result<Option<Document>, String> {
    if let Some(document) = crate::drafts::read(key) {
        return Ok(Some(document));
    }
    let json = browser::load_draft(key).map_err(crate::message)?;
    json.map(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
        .transpose()
}

pub async fn initialize() {
    if let Err(error) = migrate_catalog().await {
        WARNING.with_borrow_mut(|warning| *warning = Some(error));
    } else {
        reload_cache().await;
    }
}

pub async fn reload_cache() {
    match read_catalog().await {
        Ok(saved) => {
            CATALOG.with_borrow_mut(|catalog| *catalog = saved);
            WARNING.with_borrow_mut(|warning| *warning = None);
        }
        Err(error) => WARNING.with_borrow_mut(|warning| *warning = Some(error)),
    }
}

async fn read_catalog() -> Result<Vec<SavedProject>, String> {
    let values = storage::read_records(storage::PROJECTS).await?;
    let mut records = Vec::new();
    for value in &values {
        if let Some(saved) = serde_json::from_str::<Option<SavedProject>>(value)
            .map_err(|_| "Could not read your saved project list.")?
        {
            records.push(saved);
        }
    }
    records.sort_by_key(|saved| saved.order);
    Ok(records)
}

async fn migrate_catalog() -> Result<(), String> {
    let tx = storage::write_transaction(storage::PROJECTS).await?;
    let store = tx.object_store(storage::PROJECTS);
    let request = store.get(MIGRATED);
    request.add_event_listener(
        events::Success,
        move |_: &Event| {
            if matches!(js::classify(request.result()), js::Kind::Undefined) {
                let Ok(records) = legacy_records() else {
                    tx.abort();
                    return;
                };
                for (key, value) in records {
                    store.add(value, Some(&key));
                }
                store.add("null", Some(&MIGRATED));
            }
        },
        None,
    );
    storage::committed(
        tx,
        "Could not migrate saved projects. The original browser records are intact.",
    )
    .await
}

fn legacy_records() -> Result<Vec<(String, String)>, String> {
    let Some(raw) = browser::load_draft(LEGACY_CATALOG).map_err(crate::message)? else {
        return Ok(Vec::new());
    };
    let projects: Vec<Project> = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
    let mut records = Vec::new();
    for (index, project) in projects.into_iter().enumerate() {
        let bundle = browser::load_draft(&project.legacy_bundle_key()).map_err(crate::message)?;
        if let Some(raw) = &bundle {
            ProjectBundle::parse(raw)?;
        }
        let key = format!("project:{}", project.id);
        let value = serde_json::to_string(&SavedProject {
            project,
            bundle,
            order: index as i64,
        })
        .map_err(|error| error.to_string())?;
        records.push((key, value));
    }
    Ok(records)
}

pub fn saved_bundle(id: &str) -> Option<String> {
    CATALOG.with_borrow(|catalog| {
        catalog
            .iter()
            .find(|saved| saved.project.id == id)
            .and_then(|saved| saved.bundle.clone())
    })
}

pub fn warning() -> Option<String> {
    WARNING.with_borrow(Clone::clone)
}

pub fn list() -> (Vec<Project>, Option<String>) {
    let mut projects = vec![
        Project {
            id: ORBIT.into(),
            title: "Orbit study".into(),
        },
        Project {
            id: CURRENT.into(),
            title: "Current — Home energy".into(),
        },
        canopy_library(),
        Project {
            id: crate::journey::PROJECT.into(),
            title: "Current — Reminder journey".into(),
        },
    ];
    let legacy = blank_mockup();
    if legacy
        .load(false)
        .ok()
        .flatten()
        .is_some_and(|document| document.scene.is_some())
    {
        projects.push(legacy);
    }
    CATALOG.with_borrow(|catalog| {
        projects.extend(catalog.iter().rev().map(|saved| saved.project.clone()))
    });
    projects.extend(crate::drafts::recoveries());
    (projects, warning())
}

pub fn find(id: &str) -> Option<Project> {
    if id == WEB_APP {
        Some(blank_mockup())
    } else {
        list().0.into_iter().find(|project| project.id == id)
    }
}

pub fn canopy_library() -> Project {
    Project {
        id: CANOPY.into(),
        title: "Canopy UI — Design system".into(),
    }
}

fn blank_mockup() -> Project {
    Project {
        id: WEB_APP.into(),
        title: "Web app mockup".into(),
    }
}

pub fn route() -> Option<(String, bool)> {
    if let Some(project) = browser::query_value("project") {
        Some((project, false))
    } else if browser::workspace_query("design") {
        Some((WEB_APP.into(), false))
    } else if browser::workspace_query("server") {
        Some((ORBIT.into(), true))
    } else {
        None
    }
}

pub async fn create(title: &str) -> Result<Project, String> {
    let title = title.trim();
    if !pixel_geometry::bundle::valid_title(title) {
        return Err("Enter a name between 1 and 80 characters.".into());
    }
    publish(title, None).await
}

pub async fn restore_bundle(bundle: &ProjectBundle, json: &str) -> Result<Project, String> {
    publish(&bundle.title, Some(json)).await
}

async fn publish(title: &str, bundle: Option<&str>) -> Result<Project, String> {
    if let Some(error) = warning() {
        return Err(error);
    }
    let project = Project {
        id: crypto().random_uuid(),
        title: title.into(),
    };
    let key = format!("project:{}", project.id);
    let saved = SavedProject {
        project: project.clone(),
        bundle: bundle.map(str::to_owned),
        order: js::date::now() as i64,
    };
    let value = serde_json::to_string(&saved).map_err(|error| error.to_string())?;
    let tx = storage::write_transaction(storage::PROJECTS).await?;
    let ready = storage::committed(tx, "Could not save the new project in this browser.");
    tx.object_store(storage::PROJECTS).add(value, Some(&key));
    ready.await?;
    CATALOG.with_borrow_mut(|catalog| {
        catalog.retain(|entry| entry.project.id != project.id);
        catalog.push(saved);
        catalog.sort_by_key(|entry| entry.order);
    });
    let _ = browser::save_draft(CHANGED, Some(&crypto().random_uuid()));
    Ok(project)
}
