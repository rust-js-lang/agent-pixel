//! Project and screen URLs live in React Router's browser history.
use agent_pixel_react_router::{BrowserRouter, Navigate, Navigation, use_browser_navigation};
use js::{Promise, promise};
use react::{JSX, jsx, use_effect};
use std::cell::{Cell, RefCell};
use webapi::{
    Element, EventTargetExt, HTMLAnchorElement, PointerEvent, events, url, window_global::window,
};

thread_local! {
    static NAVIGATE: Cell<Option<&'static Navigate>> = const { Cell::new(None) };
    static READY: Cell<bool> = const { Cell::new(false) };
    static ROUTING: Cell<bool> = const { Cell::new(false) };
    static PAGE_VERSION: Cell<u32> = const { Cell::new(0) };
    static TARGET: RefCell<Option<String>> = const { RefCell::new(None) };
    static WARNING: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn start() {
    window()
        .document()
        .add_event_listener(events::Click, handle_link, None);
    crate::render_view(jsx! { <p role="status">"Opening Agent Pixel…"</p> });
    js::spawn(Box::new(async {
        let warning = crate::initialize().await;
        WARNING.with_borrow_mut(|slot| *slot = warning);
        READY.set(true);
        open_pending();
    }));
}

pub fn page(content: JSX::Element) -> JSX::Element {
    let key = PAGE_VERSION.get().to_string();
    jsx! {
        <BrowserRouter>
            <><RouteObserver /><Fragment key={key}>{content}</Fragment></>
        </BrowserRouter>
    }
}

fn RouteObserver() -> JSX::Element {
    let Navigation { location, navigate } = use_browser_navigation();
    let search = location.search.clone();
    let path = location.pathname.clone();
    use_effect(
        move || {
            NAVIGATE.set(Some(navigate));
            TARGET.with_borrow_mut(|target| *target = Some(search.clone()));
            open_pending();
        },
        Some(&[path, location.search.clone()]),
    );
    jsx! { <></> }
}

fn open_pending() {
    if !READY.get() || ROUTING.replace(true) {
        return;
    }
    js::spawn(Box::new(async {
        loop {
            while crate::EDITOR.with_borrow(|editor| editor.busy) {
                let idle: Promise<()> = promise::new(|resolve, _| {
                    js::set_timeout(move || resolve(()), Some(16.0));
                });
                idle.await;
            }
            let Some(search) = TARGET.take() else { break };
            PAGE_VERSION.set(PAGE_VERSION.get().wrapping_add(1));
            let warning = WARNING.take();
            crate::open_route(&search, warning).await;
        }
        ROUTING.set(false);
    }));
}

pub fn navigate(href: &str) {
    let location = window().location();
    let base = location.href();
    let Some(url) = url::parse(href, Some(&base)) else {
        return;
    };
    if let Some(navigate) = NAVIGATE.get() {
        if url.pathname() != location.pathname() || url.search() != location.search() {
            crate::webmcp::clear_tools();
        }
        navigate(&format!("{}{}{}", url.pathname(), url.search(), url.hash()));
    }
}

fn handle_link(event: &PointerEvent) {
    if event.default_prevented()
        || event.button() != 0.0
        || event.meta_key()
        || event.ctrl_key()
        || event.shift_key()
        || event.alt_key()
    {
        return;
    }
    let Some(link) = event
        .target()
        .and_then(js::downcast::<Element>)
        .and_then(|element| element.closest("a[href]"))
        .and_then(js::downcast::<HTMLAnchorElement>)
    else {
        return;
    };
    let location = window().location();
    if link.origin() != location.origin()
        || link.pathname() != location.pathname()
        || !link.target().is_empty()
        || link.has_attribute("download")
    {
        return;
    }
    if link.search() == location.search() && link.hash() != location.hash() {
        return;
    }
    event.prevent_default();
    if link.search() != location.search() {
        navigate(&link.href());
    }
}
