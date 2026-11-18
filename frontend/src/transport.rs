//! Request/reply protocol for the document worker.
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use js::{Kind, Promise, promise};
use pixel_geometry::scene::{Author, Scene, Session};
use serde::{Deserialize, Serialize};
use webapi::{Event, EventTargetExt, MessageEvent, Worker, WorkerOptions, events, global, worker};

const FAILURE: &str = "Document worker stopped. Reload the page.";
type Reply = Result<String, String>;
type Resolve = &'static dyn Fn(Reply);

#[derive(Deserialize, Serialize)]
struct Request {
    sequence: u32,
    command: String,
    author: Author,
    json: String,
}

#[derive(Deserialize, Serialize)]
struct Response {
    sequence: u32,
    result: Reply,
}

thread_local! {
    static WORKER: Cell<Option<&'static Worker>> = const { Cell::new(None) };
    static FAILED: Cell<bool> = const { Cell::new(false) };
    static SEQUENCE: Cell<u32> = const { Cell::new(0) };
    static PENDING: RefCell<HashMap<u32, Resolve>> = RefCell::new(HashMap::new());
    static SESSION: RefCell<Session> = RefCell::new(Session::new(Scene::poster()));
}

pub fn request(command: &str, json: &str, author: Author) -> Promise<Reply> {
    if FAILED.get() {
        return promise::resolve(Err(FAILURE.into()));
    }
    let worker = WORKER.get().unwrap_or_else(start_worker);
    let sequence = SEQUENCE.get().wrapping_add(1);
    SEQUENCE.set(sequence);
    let message = serde_json::to_string(&Request {
        sequence,
        command: command.into(),
        author,
        json: json.into(),
    })
    .expect("Serialize worker request");
    promise::new(|resolve, _| {
        PENDING.with_borrow_mut(|pending| pending.insert(sequence, resolve));
        worker.post_message(message, None);
    })
}

fn start_worker() -> &'static Worker {
    let worker = worker::new(
        "./worker.js",
        Some(WorkerOptions {
            r#type: Some("module"),
            ..WorkerOptions::default()
        }),
    );
    worker.add_event_listener(events::Message, receive_reply, None);
    worker.add_event_listener(events::Error, |_: &Event| fail_worker(), None);
    worker.add_event_listener(events::Messageerror, |_: &MessageEvent| fail_worker(), None);
    WORKER.set(Some(worker));
    worker
}

fn receive_reply(event: &MessageEvent) {
    if let Kind::String(message) = js::classify(event.data()) {
        if let Ok(response) = serde_json::from_str::<Response>(message) {
            if let Some(resolve) =
                PENDING.with_borrow_mut(|pending| pending.remove(&response.sequence))
            {
                resolve(response.result);
            }
            return;
        }
    }
    fail_worker();
}

fn fail_worker() {
    FAILED.set(true);
    let pending = PENDING.take();
    for (_, resolve) in pending {
        resolve(Err(FAILURE.into()));
    }
}

// DedicatedWorkerGlobalScope has its own postMessage and message listener.
#[cfg_attr(rust_js, rust_js::link_name = "globalThis.addEventListener")]
fn listen(event: &str, handler: fn(&MessageEvent)) {
    unreachable!()
}

#[cfg_attr(rust_js, rust_js::link_name = "globalThis.postMessage")]
fn post_reply(message: &str) {
    unreachable!()
}

pub fn serve_worker() {
    listen("message", receive_request);
}

fn receive_request(event: &MessageEvent) {
    if let Kind::String(json) = js::classify(event.data()) {
        if let Ok(request) = serde_json::from_str::<Request>(json) {
            let result = SESSION.with_borrow_mut(|session| {
                session.command(
                    &request.command,
                    &request.json,
                    global::crypto().random_uuid(),
                    request.author,
                )
            });
            let response = Response {
                sequence: request.sequence,
                result,
            };
            let json = serde_json::to_string(&response).expect("Serialize worker reply");
            post_reply(&json);
        }
    }
}
