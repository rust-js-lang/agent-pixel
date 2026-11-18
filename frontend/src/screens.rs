//! Browser persistence for project screen lists and stable routes.
use crate::{backup, browser, drafts, projects::Project, storage};
use pixel_geometry::{
    bundle::ScreenInfo,
    scene::Document,
    screens::{INITIAL_REVISION, ScreenCatalog, ScreenEdit},
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use webapi::{Event, EventTargetExt, events, global::crypto};

pub const CHANGED: &str = "agent-pixel:screens-changed";

#[derive(Clone, Deserialize, Serialize)]
struct Stored {
    project_id: String,
    catalog: ScreenCatalog,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenUpdate {
    pub catalog: ScreenCatalog,
    pub screen_id: String,
}

thread_local! {
    static CATALOGS: RefCell<Vec<Stored>> = const { RefCell::new(Vec::new()) };
}

pub async fn reload_cache() -> Result<(), String> {
    let values = storage::read_records(storage::SCREENS).await?;
    let records = values
        .iter()
        .map(|json| serde_json::from_str(json).map_err(|error| error.to_string()))
        .collect::<Result<Vec<Stored>, String>>()?;
    CATALOGS.with_borrow_mut(|catalogs| *catalogs = records);
    Ok(())
}

pub fn catalog(project: &Project, initial: Vec<ScreenInfo>) -> ScreenCatalog {
    CATALOGS
        .with_borrow(|catalogs| {
            catalogs
                .iter()
                .find(|stored| stored.project_id == project.id)
                .map(|stored| stored.catalog.clone())
        })
        .unwrap_or_else(|| ScreenCatalog::from_screens(initial))
}

pub fn stored_catalog(value: Option<&js::Unknown>) -> Result<Option<ScreenCatalog>, String> {
    match js::classify(value) {
        js::Kind::Undefined => Ok(None),
        js::Kind::String(json) => serde_json::from_str::<Stored>(json)
            .map(|stored| Some(stored.catalog))
            .map_err(|error| error.to_string()),
        _ => Err("Could not read the screen list.".into()),
    }
}

pub async fn update(
    project: &Project,
    expected: &str,
    edit: ScreenEdit,
) -> Result<ScreenUpdate, String> {
    let source = backup::source(project)?;
    let mut catalog = catalog(project, source.screen_info());
    let revision = crypto().random_uuid();
    let edited = catalog.apply(edit, expected, revision.clone())?;
    let snapshot = if let Some(id) = edited.copy_from {
        drafts::reload_cache().await?;
        let scene = source.scene(project, &catalog, &id)?;
        let key = project.screen_key(false, &catalog.legacy_base_id, &edited.id);
        let body = drafts::snapshot(
            key.clone(),
            revision,
            Document {
                revision: 0,
                scene: Some(scene),
            },
        )?;
        Some((key, body))
    } else {
        None
    };
    let record = Stored {
        project_id: project.id.clone(),
        catalog,
    };
    let json = serde_json::to_string(&record).map_err(|error| error.to_string())?;
    let tx = storage::write_screens_and_drafts().await?;
    let store = tx.object_store(storage::SCREENS);
    let key = project.id.clone();
    let revision = expected.to_owned();
    let check = store.get(&key);
    let stale = Rc::new(Cell::new(false));
    let conflict = stale.clone();
    check.add_event_listener(
        events::Success,
        move |_: &Event| {
            let current = match stored_catalog(check.result()) {
                Ok(Some(catalog)) => catalog.revision,
                Ok(None) => INITIAL_REVISION.to_owned(),
                Err(_) => {
                    tx.abort();
                    return;
                }
            };
            if current != revision {
                conflict.set(true);
                tx.abort();
                return;
            }
            store.put(&json, Some(&key));
            if let Some((key, body)) = &snapshot {
                tx.object_store(storage::DRAFTS).add(body, Some(key));
            }
        },
        None,
    );
    if let Err(error) = storage::committed(tx, "Could not save the screen list.").await {
        if stale.get() {
            reload_cache().await?;
            return Err("The screen list changed. Read its latest revision and retry.".into());
        }
        return Err(error);
    }
    let catalog = record.catalog.clone();
    CATALOGS.with_borrow_mut(|catalogs| {
        catalogs.retain(|stored| stored.project_id != record.project_id);
        catalogs.push(record);
    });
    let _ = drafts::reload_cache().await;
    let _ = browser::save_draft(CHANGED, Some(&crypto().random_uuid()));
    Ok(ScreenUpdate {
        catalog,
        screen_id: edited.id,
    })
}
