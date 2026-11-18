# Agent Pixel

A vector design editor for people and WebMCP agents. The React editor and document worker are written in Rust and compiled to JavaScript by rust-js. The optional server runs native Rust and shares the worker's document model.

[Open Agent Pixel](https://rust-js-lang.github.io/agent-pixel/) · [Read the design](DESIGN.md) · [Production roadmap](ROADMAP.md)

## Run locally

Clone [rust-js](https://github.com/rust-js-lang/rust-js) beside this project and check out `ae378c0fc55459c2fc4b4796502a74bec5cd4f5c`. Set `RUST_JS_DIR` if the checkout lives elsewhere. From the Agent Pixel repository root, link the compiler's runtime, build the browser assets and start the loopback server.

~~~sh
bun install --frozen-lockfile
bun setup-rust-js.ts
bun run build
cargo run --locked --manifest-path server/Cargo.toml
~~~

Open `http://127.0.0.1:4173/` for the project dashboard. A fresh browser starts with three editable demos, **Orbit study**, **Current — Home energy**, and **Canopy UI — Design system**. Click a card to open its editor, or name a project and choose **New project** to start an empty artboard. Project names and drafts are stored in the current browser. The optional native document service is available at `http://127.0.0.1:4173/?server`. **Save to server** writes a revision to `data/document.json`. A stale save leaves the local draft intact. **Load server copy** replaces it with the current server revision.

Select a shape on the poster or in Layers. Drag it, edit its bounds in the inspector, or change its fill. A WebMCP-capable browser exposes `read_design` and `edit_layer` while the page is open. To try an agent edit, read the scene, then call `edit_layer` with `{"id":"satellite","fill":"#247a69"}`.

## Rust source and generated JavaScript

- `frontend/src` contains the editor, React components written with `jsx!`, and the WebMCP tool definitions and handlers.
- `core/src` contains the poster, edits, bounds and validation shared by the worker and the native server.
- `frontend/src/browser.rs`, `transport.rs` and `webmcp.rs` handle platform calls, worker messages and tool registration in Rust.
- `site` contains HTML and CSS, plus the page and worker entry modules that call the compiled Rust functions.

The Pages build bundles checked-in rust-js output. To regenerate that output after a Rust change, build the compiler from its [documented setup](https://github.com/rust-js-lang/rust-js/blob/ae378c0fc55459c2fc4b4796502a74bec5cd4f5c/DEVELOPMENT.md) and then run the frontend build. rust-js also regenerates the Rust dependencies used by the frontend.

~~~sh
(cd .rust-js && bun install && bun run build:compiler)
RUST_JS_COMPILER="$PWD/.rust-js/target/debug/rust-js" bun run build:rust
bun run build
~~~

## Verify

Run `bun test` to exercise the compiled document rules and `cargo test --locked --manifest-path core/Cargo.toml` to exercise native Rust.

~~~sh
bunx --bun playwright install --with-deps --only-shell chromium
bun run test:e2e
~~~

Playwright starts the native Rust server on loopback port 4187 with a temporary document directory. It covers pointer dragging, inspector edits, reload persistence and recovery from a conflicting server save. The agent tests discover and invoke the registered tools using Playwright's `page.webmcp` API with WebMCP enabled in Chromium. The Pages workflow runs the suite and uploads traces on failure.

## Design a web app with a coding agent

Open [Web app mockup](https://rust-js-lang.github.io/agent-pixel/?design) in a Chromium session with WebMCP enabled. It opens a blank mobile artboard on the first visit and keeps its own draft in the browser.

1. Discover the tools with `page.webmcp.tools()` and call `read_design` for the dimensions and current layers.
2. Call `edit_layer` with no ID to add a rectangle, ellipse or text layer. Place background layers first because newer layers paint on top.
3. Keep the returned `edited` IDs and pass an ID back to change a layer. Read the design at the end to verify the positions, colors and content.

To place a heading, call the tool with properties like these.

~~~js
await page.webmcp.callTool("edit_layer", {
  kind: "text", text: "Your tasks", x: 24, y: 59,
  width: 380, height: 52, fontSize: 38, fill: "#202b29"
});
~~~

The [task dashboard test](e2e/design.spec.js) builds a complete mockup this way and verifies that its layers survive a reload.

## Start from the dashboard with WebMCP

Open the Agent Pixel home page in a WebMCP-enabled browser. Discover the dashboard tools with `page.webmcp.tools()`.

- `list_projects` returns the projects in this browser as an array of `{ id, title }` objects.
- `create_project` takes `{ title }` and returns the new project. Its card appears on the dashboard.
- `open_project` takes `{ id }` and navigates the current tab to that project's editor. Wait for navigation, discover the editor tools, then call `read_design` or `edit_layer`.

The [dashboard WebMCP test](e2e/dashboard-webmcp.spec.js) exercises the handoff and verifies that the project and edits survive reopening.

## Reuse Canopy UI components

In an editor, expand **Canopy UI components**, choose a variant, set its position, and select **Insert component**. Insertions create linked instances of the Canopy UI variants saved in this browser. Edit the main component in Canopy UI to update instances in open editors and when a project is reopened. Select any layer of an instance to switch its variant in the inspector. Existing independent copies remain editable.

An agent can call `list_components` for the variant IDs, then call `insert_component` with `{ id, x, y }`. The coordinates place the variant's top-left corner in artboard pixels. The result includes the new layer IDs and the updated scene. Use the instance ID from `scene.instances` with `switch_component_variant` and a variant ID from the same family.

## User journey and developer handoff

Open **Current — Reminder journey** for five editable screens of the off-peak reminder feature. Select **Play prototype** and follow the primary actions. The review screen also links to a simulated save failure. Each screen has its own saved draft. The [developer handoff](handoff/CURRENT-REMINDERS.md) documents the screen map, state rules, and acceptance criteria.

## Canvas zoom

Use **−** and **+** beside the editor toolbar, or use Ctrl/Cmd with **+** or **−**. Click the percentage or press Ctrl/Cmd+0 to return to fit. Trackpad pinch gestures that arrive as a modified wheel event and Ctrl/Cmd+wheel over the canvas zoom the artboard around the pointer. Regular scrolling pans the workspace when the artboard is larger than the viewport.

The dashboard, editor, canvas, shared artboard, journey prototype, and page shell use CSS Modules in `frontend/src`. rust-js checks the class names in each corresponding Rust binding. Bun bundles the scoped rules into `app.css`. The shared site stylesheet contains only document-wide defaults.

## Back up and restore a project

Open **Project backup** on the dashboard to download a versioned JSON bundle or restore one as a new project. On the dashboard, agents can call `export_project` with `{ id }` and `import_project` with `{ bundle }`. Each bundle contains the saved browser artboards, prototype links, and a snapshot of the component library. Restored instances resolve against that snapshot so they keep their appearance in a fresh browser.
