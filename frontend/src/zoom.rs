use react::dom::flush_sync;
use webapi::{
    AddEventListenerOptions, EventTargetExt, HTMLDivElement, KeyboardEvent, WheelEvent, events,
    window_global::document,
};

pub const MIN: f64 = 25.0;
pub const MAX: f64 = 400.0;
pub const FIT: f64 = 100.0;

const STEPS: [f64; 10] = [
    25.0, 50.0, 75.0, 100.0, 125.0, 150.0, 200.0, 250.0, 300.0, 400.0,
];

pub fn attach(node: Option<&'static HTMLDivElement>) {
    if let Some(node) = node {
        node.add_event_listener(
            events::Wheel,
            wheel,
            Some(&AddEventListenerOptions {
                passive: Some(false),
                ..AddEventListenerOptions::default()
            }),
        );
    }
}

pub fn keyboard(event: &KeyboardEvent) {
    if !(event.ctrl_key() || event.meta_key()) || event.alt_key() {
        return;
    }
    match event.key().as_str() {
        "+" | "=" => {
            event.prevent_default();
            step(true);
        }
        "-" => {
            event.prevent_default();
            step(false);
        }
        "0" => {
            event.prevent_default();
            reset();
        }
        _ => {}
    }
}

pub fn step(increase: bool) {
    let current = crate::EDITOR.with_borrow(|editor| editor.zoom);
    let next = if increase {
        STEPS
            .iter()
            .copied()
            .find(|zoom| *zoom > current + 0.5)
            .unwrap_or(MAX)
    } else {
        STEPS
            .iter()
            .rev()
            .copied()
            .find(|zoom| *zoom < current - 0.5)
            .unwrap_or(MIN)
    };
    apply(next, None);
}

pub fn reset() {
    apply(FIT, None);
}

fn wheel(event: &WheelEvent) {
    if !(event.ctrl_key() || event.meta_key()) || event.delta_y() == 0.0 {
        return;
    }
    event.prevent_default();
    let pixels = match event.delta_mode() {
        1.0 => event.delta_y() * 16.0,
        2.0 => event.delta_y() * 120.0,
        _ => event.delta_y(),
    };
    let current = crate::EDITOR.with_borrow(|editor| editor.zoom);
    apply(
        current * (-pixels * 0.004).exp(),
        Some((event.client_x(), event.client_y())),
    );
}

fn apply(next: f64, pointer: Option<(f64, f64)>) {
    let next = next.clamp(MIN, MAX);
    let current = crate::EDITOR.with_borrow(|editor| editor.zoom);
    if next == current {
        return;
    }
    let doc = document();
    let (Some(canvas), Some(art)) = (
        doc.get_element_by_id("canvas-scroll"),
        doc.get_element_by_id("art"),
    ) else {
        return;
    };
    let visible = canvas.get_bounding_client_rect();
    let before = art.get_bounding_client_rect();
    let (x, y) = pointer.unwrap_or((
        visible.left() + visible.width() / 2.0,
        visible.top() + visible.height() / 2.0,
    ));
    let fraction_x = (x - before.left()) / before.width();
    let fraction_y = (y - before.top()) / before.height();
    crate::EDITOR.with_borrow_mut(|editor| editor.zoom = next);
    flush_sync(crate::render);
    let after = art.get_bounding_client_rect();
    canvas.set_scroll_left(canvas.scroll_left() + after.left() + fraction_x * after.width() - x);
    canvas.set_scroll_top(canvas.scroll_top() + after.top() + fraction_y * after.height() - y);
}
