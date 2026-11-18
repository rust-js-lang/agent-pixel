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
- `bindings/react-router` is the Rust crate for React Router's browser router, location, and navigation hooks. The application uses React Router 8.5.1.
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

In an editor, open **Assets** and expand **Canopy UI components**, choose a variant, set its position, and select **Insert component**. Insertions create linked instances of the Canopy UI variants saved in this browser. Edit the main component in Canopy UI to update instances in open editors and when a project is reopened. Select any layer of an instance to switch its variant in the inspector. Existing independent copies remain editable.

An agent can call `list_components` for the variant IDs, then call `insert_component` with `{ id, x, y }`. The coordinates place the variant's top-left corner in artboard pixels. The result includes the new layer IDs and the updated scene. Use the instance ID from `scene.instances` with `switch_component_variant` and a variant ID from the same family.

## User journey and developer handoff

Open **Current — Reminder journey** for five editable screens of the off-peak reminder feature. Select **Play prototype** and follow the primary actions. The review screen also links to a simulated save failure. Each screen has its own saved draft. The [developer handoff](handoff/CURRENT-REMINDERS.md) documents the screen map, state rules, and acceptance criteria.

## Canvas zoom

Use **−** and **+** beside the editor toolbar, or use Ctrl/Cmd with **+** or **−**. Click the percentage or press Ctrl/Cmd+0 to return to fit. Trackpad pinch gestures that arrive as a modified wheel event and Ctrl/Cmd+wheel over the canvas zoom the artboard around the pointer. Regular scrolling pans the workspace when the artboard is larger than the viewport.

The dashboard, editor, canvas, shared artboard, journey prototype, and page shell use CSS Modules in `frontend/src`. rust-js checks the class names in each corresponding Rust binding. Bun bundles the scoped rules into `app.css`. The shared site stylesheet contains only document-wide defaults.

## Back up and restore a project

Open **Project backup** on the dashboard to download a versioned JSON bundle or restore one as a new project. On the dashboard, agents can call `export_project` with `{ id }` and `import_project` with `{ bundle }`. Each bundle contains the saved browser artboards, prototype links, and a snapshot of the component library. Restored instances resolve against that snapshot so they keep their appearance in a fresh browser.

## Client-side routing

Dashboard, project, screen, and prototype links use React Router in the same browser document. The existing query URLs and browser Back and Forward work under the GitHub Pages project prefix. WebMCP exposes the tools for the active view as you navigate.

## Project screens

Open **File** and expand **Manage screens** to add, rename, duplicate, reorder, or remove screens. Each screen has a stable URL and its own saved artboard. The bare project URL opens the first screen in the list. A duplicate includes the source's saved artboard and assets. Projects can add up to 64 screens.

Agents call `list_screens` for IDs and the screen-list revision, then `edit_screens` with that revision and an edit. The operations are `create`, `rename`, `duplicate`, `reorder`, and `delete`. Reorder uses a zero-based position. Use `open_screen` with an ID to navigate and discover the editor's tools there. Undo and redo operate on the active artboard's design edits. Screen-list changes have their own revision.

## Artboard dimensions

Open **Artboard size** in the inspector to set the width and height in artboard pixels. Each dimension can be 64–8192 px. Layers keep their coordinates when the artboard changes size. Move or resize any layers outside the proposed bounds before shrinking.

Use `{ op: "resize_artboard", width, height }` in `apply_edits` to resize alongside other edits. Read the current size from `read_design.board`. Dimensions are saved with each screen and follow it into backups and prototypes. Older projects retain the default 480 × 600 px.

## Nested frames

Use **Frame +** in the toolbar, then select the frame to edit its position, size, fill, and clipping in the inspector. Move layers or another frame into it with **Parent group**. Moving a frame moves its descendants. Resizing the frame preserves their artboard coordinates and updates the clipping boundary.

Agents use `create_frame` and `set_frame` in `apply_edits`. Both take a `frame` object with `bounds` (x, y, width, height), a six-digit hex `fill`, and a boolean `clip`. Coordinates are relative to the artboard. Use `set_layer_parent` and `read_layer_tree` for nesting. Frames and their children are included in undo, draft saves, and portable bundles.

## Canvas and layer hierarchy

The left rail toggles between **File** for screens and layers and **Assets** for image and component placement. The Design inspector stays on the right. The design toolbar floats at the bottom of the canvas, and zoom stays in the upper corner. The canvas uses a compact, outlined arrow cursor. The canvas and long panels scroll independently. Agents can open either panel with `show_workspace_panel`. On phones the Layers panel and inspector sit above and below the canvas.

Enter a name and use **+ Group** in the Layers panel. Select a layer, linked component, or group and change its **Parent group** in the inspector. Use the disclosure arrow to expand or collapse children. **Ungroup** lifts the children into the group's parent and preserves them.

Agents can call `read_layer_tree` to discover ordered nodes and stable IDs. Use `create_group`, `set_layer_parent`, `rename_group`, and `ungroup` inside `apply_edits`. To move a node to the top level, set `parentId` to `null`. Hierarchy edits are saved, reversible, and included in project bundles.

## Edit in transactions

Use **Undo**, **Redo**, and **Delete layer** in the editor. Deleting a linked component layer removes the whole instance. The inspector labels the last committed change as yours or the agent's.

For multi-layer agent edits, call `read_design` for its current revision and then `apply_edits` with `{ revision, edits }`. Each edit is `{ op: "patch", layer }` using the fields accepted by `edit_layer`, or `{ op: "delete", id }`. To place an image in the same transaction, use `{ op: "place_image", placement: { image: { name, dataUrl }, x, y, width, height } }`. A successful batch counts as one undo step. Use `undo_edit` and `redo_edit` with the latest revision. The worker keeps the last 32 transactions in the current editor session.

## Image assets

Open **Assets** on the left and use **Place image** to upload PNG, JPEG, or WebP. Agents supply the same formats as base64 data URLs in `apply_edits`. Images must be at most 512 KiB and 16 megapixels each. An artboard holds up to 1.2 MB of encoded image data, and a portable bundle holds up to 2 MB total.

Each image layer references an asset stored with its scene revision. Repeated placements of the same named file reuse that asset within the artboard. Undo, checkpoint recovery, and portable backups keep the asset data needed to reopen the design.

## Durable browser drafts

Project catalog entries and their imported bundles are committed together in IndexedDB. Older browser projects migrate on first open and keep their project IDs and original storage records.

The editor saves drafts to IndexedDB and keeps the previous saved snapshot. Existing localStorage drafts are validated and migrated when opened, with the original left in place. If a stale tab tries to save over a newer revision, it saves its edits as a separate **Recovery** project. Open that project from the editor's recovery link or the dashboard.

Agents can use `get_save_status` to check the durable revision or obtain a recovery project ID. If storage fails, the current editor keeps the changes in memory. Use **Retry draft save** or `retry_draft_save` after resolving the storage error. To recover the edits in another session, use **Download unsaved project** or `export_unsaved_project` before leaving the editor. Restore the bundle from the dashboard or with `import_project`. It includes the unsaved screen, the other saved screens, and the linked component library.

After two distinct saves, **Copy previous checkpoint** creates a separate project with the earlier snapshot of the current screen. Agents can call `recover_checkpoint` with the `storageRevision` from `get_save_status`. If the saved revision has advanced, reopen the editor before selecting a checkpoint.
