//! IndexedDB draft revisions. The read cache is refreshed after storage notifications.
use std::{cell::RefCell, rc::Rc};

use crate::{browser, projects::Project, storage};
use js::{Promise, promise};
use pixel_geometry::scene::{ComponentVariant, Document};
use serde::{Deserialize, Serialize};
use webapi::{Event, EventTargetExt, events, global};

pub const CHANGED: &str = "agent-pixel:drafts-changed";

#[derive(Clone, Deserialize, Serialize)]
struct Draft {
    key: String,
    revision: String,
    document: Document,
    previous: Option<Document>,
    #[serde(default)]
    previous_library: Option<Vec<ComponentVariant>>,
    recovery: Option<Project>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    library: Option<Vec<ComponentVariant>>,
}

pub enum Commit {
    Saved(String),
    Recovery(Project),
}

thread_local! {
    static CACHE: RefCell<Vec<Draft>> = const { RefCell::new(Vec::new()) };
}

pub async fn reload_cache() -> Result<(), String> {
    let values = storage::read_records(storage::DRAFTS).await?;
    let drafts = values
        .iter()
        .map(|value| serde_json::from_str(value).map_err(|error| error.to_string()))
        .collect::<Result<Vec<Draft>, String>>()?;
    CACHE.with_borrow_mut(|cache| *cache = drafts);
    Ok(())
}

pub fn read(key: &str) -> Option<Document> {
    CACHE.with_borrow(|cache| {
        cache
            .iter()
            .find(|record| record.key == key)
            .map(|record| record.document.clone())
    })
}

pub fn revision(key: &str) -> Option<String> {
    CACHE.with_borrow(|cache| {
        cache
            .iter()
            .find(|record| record.key == key)
            .map(|record| record.revision.clone())
    })
}

pub fn recoveries() -> Vec<Project> {
    CACHE.with_borrow(|cache| {
        cache
            .iter()
            .filter_map(|record| record.recovery.clone())
            .collect()
    })
}

pub fn recovery_library(key: &str) -> Option<Vec<ComponentVariant>> {
    CACHE.with_borrow(|cache| {
        cache
            .iter()
            .find(|record| record.key == key)
            .filter(|record| record.recovery.is_some())
            .and_then(|record| record.library.clone())
    })
}

pub fn has_previous(key: &str) -> bool {
    CACHE.with_borrow(|cache| {
        cache
            .iter()
            .any(|record| record.key == key && record.previous.is_some())
    })
}

pub fn previous(
    key: &str,
    expected: &str,
) -> Result<(Document, Option<Vec<ComponentVariant>>), String> {
    CACHE.with_borrow(|cache| {
        let record = cache
            .iter()
            .find(|record| record.key == key)
            .ok_or("No saved draft.")?;
        if record.revision != expected {
            return Err(
                "The saved draft changed. Reopen the editor to recover its current checkpoint."
                    .into(),
            );
        }
        let document = record
            .previous
            .clone()
            .ok_or("No previous saved snapshot.")?;
        Ok((document, record.previous_library.clone()))
    })
}

pub async fn commit(
    key: String,
    expected: Option<String>,
    document: Document,
    title: &str,
    library: Option<Vec<ComponentVariant>>,
) -> Result<Commit, String> {
    let tx = storage::write_transaction(storage::DRAFTS).await?;
    let store = tx.object_store(storage::DRAFTS);
    let request = store.get(key.as_str());
    let written_key = key.clone();
    let revision = global::crypto().random_uuid();
    let recovery = Project {
        id: global::crypto().random_uuid(),
        title: format!("Recovery — {title}").chars().take(80).collect(),
    };
    let ready: Promise<Result<Draft, String>> = promise::new(|resolve, _| {
        let pending = Rc::new(RefCell::new(None));
        let completed = pending.clone();
        let pending = pending.clone();
        request.add_event_listener(
            events::Success,
            move |_: &Event| {
                let current = match js::classify(request.result()) {
                    js::Kind::String(raw) => match serde_json::from_str::<Draft>(raw) {
                        Ok(draft) => Some(draft),
                        Err(_) => {
                            tx.abort();
                            return;
                        }
                    },
                    js::Kind::Undefined | js::Kind::Null => None,
                    _ => {
                        tx.abort();
                        return;
                    }
                };
                if let Some(current) = &current {
                    if current.document == document {
                        pending.replace(Some(current.clone()));
                        return;
                    }
                }
                let next = if current.as_ref().map(|current| &current.revision) == expected.as_ref()
                {
                    Draft {
                        key: written_key.clone(),
                        revision: revision.clone(),
                        document: document.clone(),
                        previous: current.as_ref().map(|draft| draft.document.clone()),
                        previous_library: current.as_ref().and_then(|draft| draft.library.clone()),
                        recovery: current.and_then(|draft| draft.recovery),
                        library: library.clone(),
                    }
                } else {
                    Draft {
                        key: recovery.storage_key(false),
                        revision: revision.clone(),
                        document: document.clone(),
                        previous: None,
                        previous_library: None,
                        recovery: Some(recovery.clone()),
                        library: library.clone(),
                    }
                };
                let Ok(value) = serde_json::to_string(&next) else {
                    tx.abort();
                    return;
                };
                store.put(value, Some(&next.key));
                pending.replace(Some(next));
            },
            None,
        );
        tx.add_event_listener(
            events::Complete,
            move |_: &Event| {
                resolve(
                    completed
                        .take()
                        .ok_or_else(|| "Draft write completed without a revision.".into()),
                );
            },
            None,
        );
        tx.add_event_listener(
            events::Abort,
            move |_: &Event| {
                resolve(Err("Browser could not save this draft.".into()));
            },
            None,
        );
    });
    let saved = ready.await?;
    let result = if saved.key != key {
        Commit::Recovery(saved.recovery.clone().expect("Recovery project"))
    } else {
        Commit::Saved(saved.revision.clone())
    };
    CACHE.with_borrow_mut(|cache| {
        cache.retain(|draft| draft.key != saved.key);
        cache.push(saved);
    });
    let _ = browser::save_draft(CHANGED, Some(&global::crypto().random_uuid()));
    Ok(result)
}
