//! Shared IndexedDB connection and record reads.
use std::cell::Cell;

use js::{Promise, promise};
use webapi::{
    Event, EventTargetExt, IDBDatabase, IDBTransaction, IDBTransactionOptions,
    IDBVersionChangeEvent, events, global,
};

const DATABASE: &str = "agent-pixel";
pub const DRAFTS: &str = "drafts";
pub const PROJECTS: &str = "projects";

thread_local! {
    static DB: Cell<Option<&'static IDBDatabase>> = const { Cell::new(None) };
}

async fn database() -> Result<&'static IDBDatabase, String> {
    if let Some(db) = DB.get() {
        return Ok(db);
    }
    let request = global::indexed_db()
        .open(DATABASE, Some(2.0))
        .map_err(crate::message)?;
    let ready: Promise<Result<&IDBDatabase, String>> = promise::new(|resolve, _| {
        request.add_event_listener(
            events::Upgradeneeded,
            move |_: &IDBVersionChangeEvent| {
                if let Some(db) = request.result().and_then(js::downcast::<IDBDatabase>) {
                    let names = db.object_store_names();
                    for store in [DRAFTS, PROJECTS] {
                        if !names.contains(store) {
                            db.create_object_store(store, None);
                        }
                    }
                }
            },
            None,
        );
        request.add_event_listener(
            events::Success,
            move |_: &IDBVersionChangeEvent| {
                resolve(
                    request
                        .result()
                        .and_then(js::downcast::<IDBDatabase>)
                        .ok_or_else(|| "Project database opened without a connection.".into()),
                );
            },
            None,
        );
        request.add_event_listener(
            events::Error,
            move |_: &Event| {
                resolve(Err("Could not open the browser project database.".into()));
            },
            None,
        );
        request.add_event_listener(
            events::Blocked,
            move |_: &IDBVersionChangeEvent| {
                resolve(Err("Another tab is blocking the project database.".into()));
            },
            None,
        );
    });
    let db = ready.await?;
    db.add_event_listener(
        events::Versionchange,
        move |_: &IDBVersionChangeEvent| {
            DB.set(None);
            db.close();
        },
        None,
    );
    DB.set(Some(db));
    Ok(db)
}

pub async fn read_records(store: &str) -> Result<Vec<String>, String> {
    let db = database().await?;
    let transaction = db.transaction(store, Some("readonly"), None);
    let request = transaction.object_store(store).get_all(None, None);
    let result: Promise<Result<Vec<String>, String>> = promise::new(|resolve, _| {
        request.add_event_listener(
            events::Success,
            move |_: &Event| {
                let records = request
                    .result()
                    .and_then(|value| js::json::stringify(value).ok().flatten())
                    .ok_or_else(|| "Could not read browser records.".to_string())
                    .and_then(|json| {
                        serde_json::from_str::<Vec<String>>(&json)
                            .map_err(|error| error.to_string())
                    });
                resolve(records);
            },
            None,
        );
        request.add_event_listener(
            events::Error,
            move |_: &Event| {
                resolve(Err("Could not read browser records.".into()));
            },
            None,
        );
    });
    result.await
}

pub async fn write_transaction(store: &str) -> Result<&'static IDBTransaction, String> {
    Ok(database().await?.transaction(
        store,
        Some("readwrite"),
        Some(IDBTransactionOptions {
            durability: Some("strict"),
        }),
    ))
}

pub fn committed(tx: &'static IDBTransaction, error: &'static str) -> Promise<Result<(), String>> {
    promise::new(|resolve, _| {
        tx.add_event_listener(
            events::Complete,
            move |_: &Event| {
                resolve(Ok(()));
            },
            None,
        );
        tx.add_event_listener(
            events::Abort,
            move |_: &Event| {
                resolve(Err(error.into()));
            },
            None,
        );
    })
}
