use crate::{Editor, edit, select, sync_to_server};
use pixel_geometry::{
    BOARD_HEIGHT, BOARD_WIDTH,
    scene::{Layer, Patch},
};
use react::{CSSProperties, JSX, event::PointerEvent, jsx};
use std::cell::RefCell;
use webapi::{HTMLButtonElement, window_global::window};

struct Drag {
    id: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    origin_x: f64,
    origin_y: f64,
    scale_x: f64,
    scale_y: f64,
}

thread_local! {
    static DRAG: RefCell<Option<Drag>> = const { RefCell::new(None) };
}

impl Drag {
    fn position(&self, event: &PointerEvent<HTMLButtonElement>) -> (i32, i32) {
        let dx = ((event.client_x() - self.origin_x) * self.scale_x).round() as i32;
        let dy = ((event.client_y() - self.origin_y) * self.scale_y).round() as i32;
        (
            self.x.saturating_add(dx).clamp(0, BOARD_WIDTH - self.width),
            self.y
                .saturating_add(dy)
                .clamp(0, BOARD_HEIGHT - self.height),
        )
    }
}

fn percent(value: i32, extent: i32) -> String {
    format!("{}%", f64::from(value) / f64::from(extent) * 100.0)
}

fn label(layer: &Layer) -> String {
    if layer.id.len() == 36 {
        format!("{} {}", layer.kind, &layer.id[..4])
    } else {
        layer.id.replace('-', " ")
    }
}

fn begin_drag(layer: &Layer, event: &PointerEvent<HTMLButtonElement>) {
    if event.button() != 0 {
        return;
    }
    let board = window()
        .document()
        .get_element_by_id("art")
        .expect("Artboard");
    let bounds = board.get_bounding_client_rect();
    event
        .current_target()
        .set_pointer_capture(event.pointer_id());
    DRAG.with_borrow_mut(|slot| {
        *slot = Some(Drag {
            id: layer.id.clone(),
            x: layer.x,
            y: layer.y,
            width: layer.width,
            height: layer.height,
            origin_x: event.client_x(),
            origin_y: event.client_y(),
            scale_x: f64::from(BOARD_WIDTH) / bounds.width(),
            scale_y: f64::from(BOARD_HEIGHT) / bounds.height(),
        })
    });
    select(layer.id.clone());
}

fn set_position(event: &PointerEvent<HTMLButtonElement>, x: i32, y: i32) {
    let style = event.current_target().style();
    style.set_left(&percent(x, BOARD_WIDTH));
    style.set_top(&percent(y, BOARD_HEIGHT));
}

fn move_drag(event: &PointerEvent<HTMLButtonElement>) {
    DRAG.with_borrow(|drag| {
        if let Some(drag) = drag {
            let (x, y) = drag.position(event);
            set_position(event, x, y);
        }
    });
}

fn end_drag(event: &PointerEvent<HTMLButtonElement>, commit: bool) {
    if let Some(drag) = DRAG.take() {
        set_position(event, drag.x, drag.y);
        let (x, y) = drag.position(event);
        if commit && (x != drag.x || y != drag.y) {
            edit(Patch {
                id: Some(drag.id),
                x: Some(x),
                y: Some(y),
                ..Patch::default()
            });
        }
    }
}

fn shape(layer: &Layer) -> JSX::Element {
    let captured = layer.clone();
    let id = layer.id.clone();
    let styles = CSSProperties::new()
        .left(percent(layer.x, BOARD_WIDTH))
        .top(percent(layer.y, BOARD_HEIGHT))
        .width(percent(layer.width, BOARD_WIDTH))
        .height(percent(layer.height, BOARD_HEIGHT))
        .background(layer.fill.clone())
        .border_radius(if layer.kind == "ellipse" { "50%" } else { "0" });
    jsx! {
        <button key={layer.id.clone()} className="shape" style={styles}
            aria-label={format!("Select {}", label(layer))}
            onClick={move |_| select(id.clone())}
            onPointerDown={move |event| begin_drag(&captured, event)}
            onPointerMove={move_drag}
            onPointerUp={|event| end_drag(event, true)}
            onPointerCancel={|event| end_drag(event, false)} />
    }
}

fn layer_button(layer: &Layer, selected: Option<&str>) -> JSX::Element {
    let id = layer.id.clone();
    let swatch = CSSProperties::new()
        .background(layer.fill.clone())
        .border_radius(if layer.kind == "ellipse" { "50%" } else { "0" });
    jsx! {
        <button key={layer.id.clone()} className="layer" aria-pressed={selected == Some(layer.id.as_str())}
            onClick={move |_| select(id.clone())}>
            <i style={swatch} />{label(layer)}
        </button>
    }
}

#[derive(Clone, Copy)]
enum Field {
    X,
    Y,
    Width,
    Height,
}

fn numeric_field(field: Field, layer: &Layer, version: u32) -> JSX::Element {
    let (name, short, previous) = match field {
        Field::X => ("x", "X", layer.x),
        Field::Y => ("y", "Y", layer.y),
        Field::Width => ("width", "W", layer.width),
        Field::Height => ("height", "H", layer.height),
    };
    let id = layer.id.clone();
    jsx! {
        <label key={format!("{id}-{name}-{previous}-{version}")}>{short}
            <input type="number" name={name} aria-label={name} defaultValue={previous}
                onBlur={move |event| {
                    let input = event.current_target();
                    let value = input.value_as_number();
                    if !value.is_finite() || value.fract() != 0.0 || value < 0.0 || value > f64::from(BOARD_HEIGHT) {
                        crate::status("Keep the entire shape inside the artboard.".into());
                        input.set_value(&previous.to_string());
                    } else if value as i32 != previous {
                        let mut patch = Patch { id: Some(id.clone()), ..Patch::default() };
                        let next = Some(value as i32);
                        match field { Field::X => patch.x = next, Field::Y => patch.y = next, Field::Width => patch.width = next, Field::Height => patch.height = next }
                        edit(patch);
                    }
                }} />
        </label>
    }
}

fn inspector(layer: &Layer, version: u32) -> JSX::Element {
    let id = layer.id.clone();
    let fields = [Field::X, Field::Y, Field::Width, Field::Height]
        .into_iter()
        .map(|field| numeric_field(field, layer, version))
        .collect::<Vec<_>>();
    jsx! {
        <>
            <h2 id="name">{label(layer)}</h2><hr /><h3>"Geometry"</h3>
            <form id="fields" onSubmit={|event| event.prevent_default()}>{fields}</form>
            <h3>"Fill"</h3>
            <label className="color">
                <input type="color" id="fill" value={layer.fill.clone()}
                    onChange={move |event| edit(Patch {
                        id: Some(id.clone()), fill: Some(event.current_target().value()), ..Patch::default()
                    })} />
                <output id="hex">{layer.fill.clone()}</output>
            </label><hr />
        </>
    }
}

pub(crate) fn editor(editor: &Editor) -> JSX::Element {
    let layers = editor
        .document
        .scene
        .as_ref()
        .map_or(&[][..], |scene| scene.layers.as_slice());
    let selected = editor.selected.as_deref();
    let shape_list = layers.iter().map(shape).collect::<Vec<_>>();
    let layer_list = layers
        .iter()
        .rev()
        .map(|layer| layer_button(layer, selected))
        .collect::<Vec<_>>();
    let properties = layers
        .iter()
        .find(|layer| Some(layer.id.as_str()) == selected)
        .map(|layer| inspector(layer, editor.fields_version));
    let disabled = editor.busy || editor.document.scene.is_none();
    jsx! {
        <>
            <header>
                <b className="brand"><i>"▦"</i>" Agent Pixel"</b>
                <span className="crumb">"Workspace / "<strong>"Orbit study"</strong></span>
                <a href="https://github.com/rust-js-lang/agent-pixel/blob/main/DESIGN.md">"Design doc ↗"</a>
            </header>
            <main>
                <aside>
                    <div className="eyebrow">"DOCUMENT"</div><h2>"Orbit study"</h2>
                    <p>"Poster composition"</p><hr /><h3>"Layers"</h3><div id="layers">{layer_list}</div>
                </aside>
                <section className="workspace" aria-label="Canvas">
                    <div className="sheet">
                        <div className="legend">"ARTBOARD 01" <span>{format!("{BOARD_WIDTH} × {BOARD_HEIGHT}")}</span></div>
                        <div id="art">
                            <div className="imprint">"FORM STUDIES" <span>"SERIES 01 / 03"</span></div>
                            <div id="shapes">{shape_list}</div>
                            <div className="caption"><b>"Orbit study."</b><small>"KINETIC FORMS" <span>"FIG. 01"</span></small></div>
                        </div>
                    </div>
                    <div className="tools">
                        <button id="rect" disabled={disabled} onClick={|_| add("rectangle")}>"▢  Rectangle +"</button>
                        <button id="ellipse" disabled={disabled} onClick={|_| add("ellipse")}>"○  Ellipse +"</button>
                    </div>
                </section>
                <aside className="properties">
                    <div className="eyebrow">"INSPECTOR"</div>
                    {properties}<span id="agent">{editor.agent.clone()}</span>
                    {editor.server.then(|| jsx! {
                        <div id="server">
                            <button disabled={disabled} onClick={|_| sync_to_server(false)}>"Save to server"</button>
                            <button disabled={disabled} onClick={|_| sync_to_server(true)}>"Load server copy"</button>
                        </div>
                    })}
                </aside>
            </main>
            <footer role="status" id="status">{editor.status.clone()}</footer>
        </>
    }
}

fn add(kind: &str) {
    edit(Patch {
        kind: Some(kind.into()),
        ..Patch::default()
    });
}
