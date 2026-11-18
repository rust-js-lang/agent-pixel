use crate::{Editor, edit, journey, select, sync_to_server, zoom};
use crate::{
    artboard_styles::STYLES as ARTBOARD, canvas_styles::STYLES as CANVAS,
    editor_styles::STYLES as EDITOR_STYLES, journey_styles::STYLES as JOURNEY,
    shell_styles::STYLES as SHELL,
};

use pixel_geometry::{
    Dimensions, MAX_ARTBOARD_SIZE, MIN_ARTBOARD_SIZE,
    assets::ImageAsset,
    hierarchy::{LayerGroup, LayerNode},
    scene::{Author, ComponentInstance, Edit, Layer, Patch, Scene},
};
use react::{CSSProperties, JSX, event::PointerEvent, jsx, use_state};
use std::cell::RefCell;
use webapi::{FormDataEntryValue, HTMLButtonElement, window_global::window};

struct Drag {
    id: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    origin_x: f64,
    origin_y: f64,
    board: Dimensions,
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
            self.x
                .saturating_add(dx)
                .clamp(0, self.board.width - self.width),
            self.y
                .saturating_add(dy)
                .clamp(0, self.board.height - self.height),
        )
    }
}

fn percent(value: i32, extent: i32) -> String {
    format!("{}%", f64::from(value) / f64::from(extent) * 100.0)
}

fn begin_drag(layer: &Layer, dimensions: Dimensions, event: &PointerEvent<HTMLButtonElement>) {
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
            board: dimensions,
            scale_x: f64::from(dimensions.width) / bounds.width(),
            scale_y: f64::from(dimensions.height) / bounds.height(),
        })
    });
    select(layer.id.clone());
}

fn set_position(event: &PointerEvent<HTMLButtonElement>, x: i32, y: i32, board: Dimensions) {
    let style = event.current_target().style();
    style.set_left(&percent(x, board.width));
    style.set_top(&percent(y, board.height));
}

fn move_drag(event: &PointerEvent<HTMLButtonElement>) {
    DRAG.with_borrow(|drag| {
        if let Some(drag) = drag {
            let (x, y) = drag.position(event);
            set_position(event, x, y, drag.board);
        }
    });
}

fn end_drag(event: &PointerEvent<HTMLButtonElement>, commit: bool) {
    if let Some(drag) = DRAG.take() {
        set_position(event, drag.x, drag.y, drag.board);
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

fn layer_style(layer: &Layer, board: Dimensions) -> CSSProperties {
    CSSProperties::new()
        .left(percent(layer.x, board.width))
        .top(percent(layer.y, board.height))
        .width(percent(layer.width, board.width))
        .height(percent(layer.height, board.height))
        .background(if layer.kind == "text" {
            "transparent".into()
        } else {
            layer.fill.clone()
        })
        .border_radius(if layer.kind == "ellipse" { "50%" } else { "0" })
        .color(layer.fill.clone())
        .font_size(layer.font_size.map_or_else(String::new, |size| {
            format!("{}cqw", f64::from(size) / f64::from(board.width) * 100.0)
        }))
}

fn layer_content(layer: &Layer, image: Option<&ImageAsset>) -> JSX::Element {
    if let Some(asset) = image {
        let style = CSSProperties::new()
            .display("block")
            .width("100%")
            .height("100%")
            .object_fit("cover")
            .pointer_events("none");
        jsx! { <img src={asset.image.data_url.clone()} alt={asset.image.name.clone()} draggable={false} style={style} /> }
    } else {
        jsx! { <>{layer.text.clone()}</> }
    }
}

pub fn preview_layer(layer: &Layer, image: Option<&ImageAsset>, board: Dimensions) -> JSX::Element {
    let class = if layer.kind == "text" {
        format!("{} {}", ARTBOARD.preview_layer(), ARTBOARD.preview_text())
    } else {
        ARTBOARD.preview_layer()
    };
    jsx! { <div key={layer.id.clone()} className={class} style={layer_style(layer, board)}>{layer_content(layer, image)}</div> }
}

fn shape(
    layer: &Layer,
    linked: bool,
    image: Option<&ImageAsset>,
    board: Dimensions,
) -> JSX::Element {
    let captured = layer.clone();
    let id = layer.id.clone();
    let styles = layer_style(layer, board);
    let class = if layer.kind == "text" {
        format!("{} {}", EDITOR_STYLES.shape(), EDITOR_STYLES.text_layer())
    } else {
        EDITOR_STYLES.shape()
    };
    jsx! {
        <button key={layer.id.clone()} className={class} style={styles}
            aria-label={format!("Select {}", image.map_or_else(|| layer.name(), |asset| asset.image.name.clone()))}
            onClick={move |_| select(id.clone())}
            onPointerDown={move |event| { if !linked { begin_drag(&captured, board, event); } }}
            onPointerMove={move_drag}
            onPointerUp={|event| end_drag(event, true)}
            onPointerCancel={|event| end_drag(event, false)}>{layer_content(layer, image)}</button>
    }
}

struct LayerRowProps {
    node: LayerNode,
    selected: Option<String>,
}

fn LayerRow(LayerRowProps { node, selected }: LayerRowProps) -> JSX::Element {
    let (expanded, set_expanded) = use_state(|| true);
    let expanded = *expanded;
    let container = node.kind == "group" || node.kind == "instance";
    let id = node.id.clone();
    let name = node.name.clone();
    let icon = match node.kind.as_str() {
        "group" => "▱",
        "instance" => "◇",
        "text" => "T",
        "image" => "▧",
        "ellipse" => "○",
        _ => "□",
    };
    let children = node
        .children
        .into_iter()
        .map(|child| {
            jsx! {
                <LayerRow key={child.id.clone()} node={child} selected={selected.clone()} />
            }
        })
        .collect::<Vec<_>>();
    jsx! {
        <li id={format!("tree-{}", node.id)}>
            <div>
                {container.then(|| jsx! {
                    <button type="button" aria-label={format!("{} {}", if expanded { "Collapse" } else { "Expand" }, name)}
                        aria-expanded={expanded} onClick={move |_| set_expanded.set(!expanded)}>{if expanded { "▾" } else { "▸" }}</button>
                })}
                <button type="button" className={EDITOR_STYLES.layer()} aria-pressed={selected.as_deref() == Some(node.id.as_str())}
                    title={name.clone()} onClick={move |_| select(id.clone())}><span aria-hidden="true">{icon}</span>{name}</button>
            </div>
            {(expanded && !children.is_empty()).then(|| jsx! { <ul>{children}</ul> })}
        </li>
    }
}

fn layer_panel(scene: Option<&Scene>, selected: Option<&str>, disabled: bool) -> JSX::Element {
    let rows = scene.map(|scene| scene.layer_tree()).unwrap_or_default();
    let nodes = rows.into_iter().map(|node| jsx! {
        <LayerRow key={node.id.clone()} node={node} selected={selected.map(str::to_owned)} />
    }).collect::<Vec<_>>();
    jsx! {
        <div id="layers" className={EDITOR_STYLES.layers()}>
            <form aria-label="New layer group" action={|data: &webapi::FormData| {
                if let Some(FormDataEntryValue::Str(name)) = data.get("group-name") {
                    crate::apply_edit(Edit::CreateGroup { name: name.trim().into(), parent_id: None });
                }
            }}>
                <input name="group-name" aria-label="New group name" placeholder="Group name" required maxLength={80} disabled={disabled} />
                <button type="submit" disabled={disabled} aria-label="Add layer group">"+ Group"</button>
            </form>
            <ul aria-label="Layer hierarchy">{nodes}</ul>
        </div>
    }
}

fn parent_picker(scene: &Scene, id: &str, disabled: bool) -> JSX::Element {
    let id = id.to_owned();
    let value = scene.parent_group(&id).unwrap_or("").to_owned();
    let options = scene.groups.iter().filter(|group| scene.is_valid_parent(&id, &group.id))
        .map(|group| jsx! { <option key={group.id.clone()} value={group.id.clone()}>{group.name.clone()}</option> })
        .collect::<Vec<_>>();
    jsx! {
        <label>"Parent group"
            <select value={value} disabled={disabled} onChange={move |event| {
                let value = event.current_target().value();
                crate::apply_edit(Edit::SetLayerParent { id: id.clone(), parent_id: (!value.is_empty()).then_some(value) });
            }}>
                <option value="">"Top level"</option>{options}
            </select>
        </label>
    }
}

fn group_inspector(group: &LayerGroup, disabled: bool) -> JSX::Element {
    let id = group.id.clone();
    let rename_id = id.clone();
    jsx! {
        <>
            <h2>{group.name.clone()}</h2>
            <form key={format!("{}-{}", group.id, group.name)} aria-label="Rename group" action={move |data: &webapi::FormData| {
                if let Some(FormDataEntryValue::Str(name)) = data.get("group-name") {
                    crate::apply_edit(Edit::RenameGroup { id: rename_id.clone(), name: name.trim().into() });
                }
            }}>
                <input name="group-name" aria-label="Group name" defaultValue={group.name.clone()} required maxLength={80} disabled={disabled} />
                <button type="submit" disabled={disabled}>"Rename"</button>
            </form>
            <p><button type="button" disabled={disabled} onClick={move |_| crate::apply_edit(Edit::Ungroup { id: id.clone() })}>"Ungroup"</button></p>
        </>
    }
}

#[derive(Clone, Copy)]
enum Field {
    X,
    Y,
    Width,
    Height,
    FontSize,
}

fn numeric_field(field: Field, layer: &Layer, version: u32, board: Dimensions) -> JSX::Element {
    let (name, short, previous, max) = match field {
        Field::X => ("x", "X", layer.x, board.width),
        Field::Y => ("y", "Y", layer.y, board.height),
        Field::Width => ("width", "W", layer.width, board.width),
        Field::Height => ("height", "H", layer.height, board.height),
        Field::FontSize => ("font size", "Px", layer.font_size.unwrap_or(20), 96),
    };
    let id = layer.id.clone();
    jsx! {
        <label key={format!("{id}-{name}-{previous}-{version}")}>{short}
            <input type="number" name={name} aria-label={name} defaultValue={previous}
                onBlur={move |event| {
                    let input = event.current_target();
                    let value = input.value_as_number();
                    if !value.is_finite() || value.fract() != 0.0 || value < 0.0 || value > f64::from(max) {
                        crate::status("Keep the entire shape inside the artboard.".into());
                        input.set_value(&previous.to_string());
                    } else if value as i32 != previous {
                        let mut patch = Patch { id: Some(id.clone()), ..Patch::default() };
                        let next = Some(value as i32);
                        match field { Field::X => patch.x = next, Field::Y => patch.y = next, Field::Width => patch.width = next, Field::Height => patch.height = next, Field::FontSize => patch.font_size = next }
                        edit(patch);
                    }
                }} />
        </label>
    }
}

fn inspector(
    layer: &Layer,
    version: u32,
    image: Option<&ImageAsset>,
    board: Dimensions,
) -> JSX::Element {
    let id = layer.id.clone();
    let fields = [Field::X, Field::Y, Field::Width, Field::Height]
        .into_iter()
        .map(|field| numeric_field(field, layer, version, board))
        .collect::<Vec<_>>();
    let text_id = layer.id.clone();
    let text = layer.text.as_ref().map(|text| jsx! {
        <>
            <h3>"Text"</h3>
            <label className={EDITOR_STYLES.text_field()}>
                <input key={format!("{text_id}-{text}-{version}")} aria-label="Text content"
                    defaultValue={text.clone()} onBlur={move |event| edit(Patch { id: Some(text_id.clone()), text: Some(event.current_target().value()), ..Patch::default() })} />
            </label>
            {numeric_field(Field::FontSize, layer, version, board)}
        </>
    });
    jsx! {
        <>
            <h2 id="name">{layer.name()}</h2><hr /><h3>"Geometry"</h3>
            <form id="fields" onSubmit={|event| event.prevent_default()}>{fields}</form>
            {text}
            {if let Some(asset) = image { jsx! { <p>{asset.image.name.clone()}</p> } } else { jsx! { <>
            <h3>{if layer.kind == "text" { "Color" } else { "Fill" }}</h3>
            <label className={EDITOR_STYLES.color()}>
                <input type="color" id="fill" value={layer.fill.clone()}
                    onChange={move |event| edit(Patch {
                        id: Some(id.clone()), fill: Some(event.current_target().value()), ..Patch::default()
                    })} />
                <output id="hex">{layer.fill.clone()}</output>
            </label></> } }}<hr />
        </>
    }
}

fn instance_inspector(instance: &ComponentInstance) -> JSX::Element {
    let id = instance.id.clone();
    let options = crate::components::variants()
        .iter()
        .filter(|variant| variant.family == instance.family)
        .map(|variant| jsx! { <option key={variant.id} value={variant.id}>{variant.name}</option> })
        .collect::<Vec<_>>();
    jsx! {
        <div className={EDITOR_STYLES.component_library()}>
            <h2>{instance.family.clone()}</h2>
            <p>"Linked to Canopy UI"</p>
            <label htmlFor="instance-variant">"Instance variant"</label>
            <select id="instance-variant" value={instance.variant.clone()} onChange={move |event| {
                let id = id.clone();
                let variant = event.current_target().value();
                js::spawn(Box::new(async move {
                    if let Err(error) = crate::change_variant(id, &variant, Author::Person).await {
                        crate::status(error);
                    }
                }));
            }}>{options}</select>
            <p><a href="?project=canopy-ui">"Open Canopy UI"</a></p>
        </div>
    }
}

pub fn board_ratio(board: Dimensions) -> f64 {
    f64::from(board.width) / f64::from(board.height)
}

pub fn board_aspect(board: Dimensions) -> String {
    format!("{} / {}", board.width, board.height)
}

fn artboard_size(board: Dimensions, disabled: bool) -> JSX::Element {
    jsx! {
        <details className={EDITOR_STYLES.component_library()}>
            <summary>"Artboard size"</summary>
            <form key={format!("{}-{}", board.width, board.height)} aria-label="Resize artboard" action={|data: &webapi::FormData| {
                if let (Some(FormDataEntryValue::Str(width)), Some(FormDataEntryValue::Str(height))) = (data.get("width"), data.get("height")) {
                    if let (Ok(width), Ok(height)) = (width.parse::<i32>(), height.parse::<i32>()) {
                        crate::apply_edit(Edit::ResizeArtboard { width, height });
                    }
                }
            }}>
                <div className={EDITOR_STYLES.component_position()}>
                    <label>"Width"<input aria-label="Artboard width" name="width" type="number" min={MIN_ARTBOARD_SIZE} max={MAX_ARTBOARD_SIZE} defaultValue={board.width} disabled={disabled} required /></label>
                    <label>"Height"<input aria-label="Artboard height" name="height" type="number" min={MIN_ARTBOARD_SIZE} max={MAX_ARTBOARD_SIZE} defaultValue={board.height} disabled={disabled} required /></label>
                </div>
                <button type="submit" disabled={disabled}>"Resize artboard"</button>
            </form>
        </details>
    }
}

fn component_picker(disabled: bool, board: Dimensions) -> JSX::Element {
    let options = crate::components::variants().iter().map(|variant| jsx! {
        <option key={variant.id} value={variant.id}>{format!("{} · {}", variant.family, variant.name)}</option>
    }).collect::<Vec<_>>();
    jsx! {
        <details className={EDITOR_STYLES.component_library()}>
            <summary>"Canopy UI components"</summary>
            <form action={|data: &webapi::FormData| {
                use webapi::FormDataEntryValue::Str;
                if let (Some(Str(id)), Some(Str(x)), Some(Str(y))) = (data.get("variant"), data.get("x"), data.get("y")) {
                    if let (Ok(x), Ok(y)) = (x.parse::<i32>(), y.parse::<i32>()) {
                        let id = id.to_string();
                        js::spawn(Box::new(async move {
                            if let Err(error) = crate::place_component(&id, x, y, Author::Person).await {
                                crate::status(error);
                            }
                        }));
                    }
                }
            }}>
                <label htmlFor="component-variant">"Component variant"</label>
                <select id="component-variant" name="variant">{options}</select>
                <div className={EDITOR_STYLES.component_position()}>
                    <label>"X"<input aria-label="Component X" name="x" type="number" min={0} max={board.width - 1} defaultValue={40} required /></label>
                    <label>"Y"<input aria-label="Component Y" name="y" type="number" min={0} max={board.height - 1} defaultValue={40} required /></label>
                </div>
                <button type="submit" disabled={disabled}>"Insert component"</button>
            </form>
        </details>
    }
}

fn journey_navigation(
    editor: &Editor,
    current: &pixel_geometry::bundle::ScreenInfo,
    preview: bool,
) -> JSX::Element {
    let screens = editor.screens.iter().map(|screen| jsx! {
        <a key={screen.id.clone()} href={journey::href(&editor.project.id, &screen.id, preview)}
            aria-current={if screen.id == current.id { "step" } else { "false" }}>{screen.title.clone()}</a>
    }).collect::<Vec<_>>();
    let name = if preview {
        "Edit screen"
    } else {
        "Play prototype"
    };
    jsx! {
        <nav className={if preview { JOURNEY.journey_navigation() } else { format!("{} {}", JOURNEY.journey_navigation(), JOURNEY.editor_navigation()) }} aria-label="Journey screens">
            <h3>"Journey screens"</h3>
            <div className={JOURNEY.journey_steps()}>{screens}</div>
            <div className={JOURNEY.journey_actions()}>
                {current.action.is_some().then(|| jsx! { <a className={JOURNEY.journey_mode()} href={journey::href(&editor.project.id, &current.id, !preview)}>{name}</a> })}
                {editor.handoff.as_ref().map(|url| jsx! { <a href={url.clone()}>"Developer handoff ↗"</a> })}
            </div>
        </nav>
    }
}

pub(crate) fn prototype(editor: &Editor) -> JSX::Element {
    let screen = editor.screen.as_ref().expect("Prototype screen");
    let scene = editor.document.scene.as_ref();
    let board = scene.map(|scene| scene.board).unwrap_or_default();
    let layers = scene.map(|scene| {
        scene
            .layers
            .iter()
            .map(|layer| preview_layer(layer, scene.image_for_layer(layer), board))
            .collect::<Vec<_>>()
    });
    let action = screen.action.as_ref().expect("Prototype action");
    let source = scene.and_then(|scene| {
        scene
            .layers
            .iter()
            .find(|layer| layer.id == action.layer_id)
    });
    let hotspot = source.map(|layer| {
        let placement = CSSProperties::new().left(percent(layer.x, board.width))
            .top(percent(layer.y, board.height)).width(percent(layer.width, board.width))
            .height(percent(layer.height, board.height));
        jsx! {
            <a className={JOURNEY.prototype_hotspot()} href={journey::href(&editor.project.id, &action.next, true)} style={placement}
                aria-label={action.label.clone()} />
        }
    });
    jsx! {
        <>
            <header className={SHELL.header()}>
                <b className={SHELL.brand()}><i>"▦"</i>" Agent Pixel"</b>
                <a href="./">"← Projects"</a>
            </header>
            <main className={JOURNEY.journey_preview()}>
                <section className={JOURNEY.prototype_canvas()} aria-label="Prototype artboard">
                    <h1>{screen.title.clone()}</h1>
                    <div id="art" style={CSSProperties::new().aspect_ratio(board_aspect(board))} className={format!("{} {}", ARTBOARD.artboard(), ARTBOARD.mockup())}>
                        <div aria-hidden="true">{layers}</div>
                        {hotspot}
                    </div>
                </section>
                <div className={JOURNEY.prototype_guide()}>
                    {journey_navigation(editor, screen, true)}
                    {action.failure.as_ref().map(|failure| jsx! {
                        <a href={journey::href(&editor.project.id, failure, true)} className={JOURNEY.simulate_failure()}>"Simulate save failure"</a>
                    })}
                </div>
            </main>
            <footer className={SHELL.footer()} role="status" id="status">{editor.status.clone()}</footer>
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
    let scene = editor.document.scene.as_ref();
    let board = scene.map(|scene| scene.board).unwrap_or_default();
    let shape_list = layers
        .iter()
        .map(|layer| {
            shape(
                layer,
                scene
                    .and_then(|scene| scene.instance_for_layer(&layer.id))
                    .is_some(),
                scene.and_then(|scene| scene.image_for_layer(layer)),
                board,
            )
        })
        .collect::<Vec<_>>();
    let disabled = editor.busy || editor.recovery.is_some() || scene.is_none();
    let selected_layer = layers
        .iter()
        .find(|layer| Some(layer.id.as_str()) == selected);
    let group = scene.and_then(|scene| {
        scene
            .groups
            .iter()
            .find(|group| Some(group.id.as_str()) == selected)
    });
    let instance = scene.and_then(|scene| {
        scene
            .instances
            .iter()
            .find(|instance| Some(instance.id.as_str()) == selected)
            .or_else(|| selected_layer.and_then(|layer| scene.instance_for_layer(&layer.id)))
    });
    let properties = if let Some(group) = group {
        Some(group_inspector(group, disabled))
    } else if let Some(instance) = instance {
        Some(instance_inspector(instance))
    } else {
        selected_layer.map(|layer| {
            inspector(
                layer,
                editor.fields_version,
                scene.and_then(|scene| scene.image_for_layer(layer)),
                board,
            )
        })
    };
    let parent = instance.map(|instance| instance.id.as_str()).or(selected);
    let parent_control = scene
        .filter(|scene| !scene.groups.is_empty())
        .and_then(|scene| parent.map(|id| parent_picker(scene, id, disabled)));
    let document_title = editor.project.title.as_str();
    let screen = editor.screen.as_ref();
    let poster = editor.poster;
    let scale = editor.zoom / 100.0;
    let ratio = board_ratio(board);
    let width = board.width;
    let sheet_style = CSSProperties::new().width(format!(
        "calc(min(var(--available-width), {width}px, calc(var(--fit-height) * {ratio})) * {scale})"
    ));
    let zoom_label = format!("{:.0}%", editor.zoom);
    jsx! {
        <div className={EDITOR_STYLES.editor()}>
            <header className={SHELL.header()}>
                <b className={SHELL.brand()}><i>"▦"</i>" Agent Pixel"</b>
                <span className={SHELL.crumb()}>"Workspace / "<strong>{document_title}</strong></span>
                <a href="./">"← Projects"</a>
                <a href="https://github.com/rust-js-lang/agent-pixel/blob/main/DESIGN.md">"Design doc ↗"</a>
            </header>
            <main className={EDITOR_STYLES.layout()}>
                <aside aria-label="Layers panel">
                    <div className={EDITOR_STYLES.eyebrow()}>"DOCUMENT"</div><h2>{document_title}</h2>
                    <p>{if poster { "Poster composition" } else { "Artboard" }}</p>
                    {screen.filter(|screen| screen.action.is_some() || editor.screens.len() > 1).map(|screen| journey_navigation(editor, screen, false))}<hr /><h3>"Layers"</h3>{layer_panel(scene, selected, disabled)}
                </aside>
                <section className={EDITOR_STYLES.workspace()} aria-label="Canvas">
                    <div id="canvas-scroll" className={CANVAS.canvas_scroll()} ref={zoom::attach}>
                        <div className={CANVAS.canvas_stage()}>
                    <div style={sheet_style}>
                        <div className={EDITOR_STYLES.legend()}>{screen.map_or("ARTBOARD 01", |screen| screen.title.as_str())} <span>{format!("{} × {}", board.width, board.height)}</span></div>
                        <div id="art" style={CSSProperties::new().aspect_ratio(board_aspect(board))} className={if poster { ARTBOARD.artboard() } else { format!("{} {}", ARTBOARD.artboard(), ARTBOARD.mockup()) }}>
                            {(poster).then(|| jsx! { <div className={EDITOR_STYLES.imprint()}>"FORM STUDIES" <span>"SERIES 01 / 03"</span></div> })}
                            <div id="shapes" className={EDITOR_STYLES.shapes()}>{shape_list}</div>
                            {(poster).then(|| jsx! { <div className={EDITOR_STYLES.caption()}><b>"Orbit study."</b><small>"KINETIC FORMS" <span>"FIG. 01"</span></small></div> })}
                        </div>
                    </div>
                        </div>
                    </div>
                    <div className={CANVAS.canvas_toolbar()}>
                    <div className={EDITOR_STYLES.tools()}>
                        <button type="button" disabled={disabled || editor.history.undo.is_empty()} onClick={|_| crate::history_action(true)}>"Undo"</button>
                        <button type="button" disabled={disabled || editor.history.redo.is_empty()} onClick={|_| crate::history_action(false)}>"Redo"</button>
                        <button type="button" disabled={disabled || selected.is_none() || group.is_some()} onClick={|_| crate::delete_selection()}>"Delete layer"</button>
                        <button id="rect" disabled={disabled} onClick={|_| add("rectangle")}>"▢  Rectangle +"</button>
                        <button id="ellipse" disabled={disabled} onClick={|_| add("ellipse")}>"○  Ellipse +"</button>
                        <button disabled={disabled} onClick={move |_| edit(Patch { kind: Some("text".into()), text: Some("Label".into()), font_size: Some(20), width: Some(170.min(board.width)), height: Some(32), fill: Some("#20231f".into()), ..Patch::default() })}>"T  Text +"</button>
                    </div>
                    <div className={CANVAS.zoom_tools()} role="group" aria-label="Canvas zoom">
                        <button type="button" aria-label="Zoom out" disabled={editor.zoom <= zoom::MIN} onClick={|_| zoom::step(false)}>"−"</button>
                        <button className={CANVAS.zoom_value()} type="button" title="Reset canvas zoom" aria-label={format!("Reset canvas zoom, current {zoom_label}")} onClick={|_| zoom::reset()}>{zoom_label}</button>
                        <button type="button" aria-label="Zoom in" disabled={editor.zoom >= zoom::MAX} onClick={|_| zoom::step(true)}>"+"</button>
                    </div>
                    </div>
                </section>
                <aside aria-label="Inspector" className={EDITOR_STYLES.properties()}>
                    <div className={EDITOR_STYLES.eyebrow()}>"INSPECTOR"</div>
                    <p><label>"Place image"
                        <input type="file" accept="image/png,image/jpeg,image/webp" disabled={disabled} onChange={|event| {
                            let input = event.current_target();
                            if let Some(file) = input.files().and_then(|files| files.item(0)) {
                                input.set_value("");
                                crate::add_image(file);
                            }
                        }} />
                    </label></p>
                    {component_picker(disabled, board)}
                    {artboard_size(board, disabled)}
                    {properties}
                    {parent_control}
                    {editor.history.undo.last().map(|entry| jsx! {
                        <p aria-label="Last edit">{format!("{} · {}", entry.label, if matches!(entry.author, Author::Agent) { "Agent" } else { "You" })}</p>
                    })}
                    {editor.recovery.as_ref().map(|project| jsx! {
                        <p><a href={project.href()}>"Open recovery project"</a></p>
                    })}
                    {(crate::drafts::has_previous(&editor.storage_key()) && !editor.busy).then(|| jsx! {
                        <button type="button" onClick={|_| {
                            let revision = crate::EDITOR.with_borrow(|editor| editor.storage_revision.clone());
                            if let Some(revision) = revision {
                                js::spawn(Box::new(async move {
                                    match crate::recover_previous(&revision).await {
                                        Ok(project) => window().location().assign(&project.href()),
                                        Err(error) => crate::status(error),
                                    }
                                }));
                            }
                        }}>"Copy previous checkpoint"</button>
                    })}
                    {(editor.save_failed && editor.recovery.is_none()).then(|| jsx! {
                        <>
                            <p>"Download a copy before leaving to keep your unsaved edits."</p>
                            <button type="button" disabled={editor.busy} onClick={|_| {
                                if let Err(error) = crate::backup::download_unsaved() {
                                    crate::status(error);
                                }
                            }}>"Download unsaved project"</button>
                            <button type="button" disabled={editor.busy} onClick={|_| {
                                js::spawn(Box::new(async { let _ = crate::retry_save().await; }));
                            }}>"Retry draft save"</button>
                        </>
                    })}
                    <span id="agent" className={EDITOR_STYLES.agent()}>{editor.agent.clone()}</span>
                    {editor.server.then(|| jsx! {
                        <div id="server" className={EDITOR_STYLES.server()}>
                            <button disabled={disabled} onClick={|_| sync_to_server(false)}>"Save to server"</button>
                            <button disabled={disabled} onClick={|_| sync_to_server(true)}>"Load server copy"</button>
                        </div>
                    })}
                </aside>
            </main>
            <footer className={SHELL.footer()} role="status" id="status">{editor.status.clone()}</footer>
        </div>
    }
}

fn add(kind: &str) {
    edit(Patch {
        kind: Some(kind.into()),
        ..Patch::default()
    });
}
