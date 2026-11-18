# Agent Pixel

A vector design editor for people and WebMCP agents. The React editor and document worker are written in Rust and compiled to JavaScript by rust-js. The optional server runs native Rust and shares the worker's document model.

[Open the editor](https://rust-js-lang.github.io/agent-pixel/) · [Read the design](DESIGN.md)

## Run locally

Clone [rust-js](https://github.com/rust-js-lang/rust-js) beside this project and check out `ae378c0fc55459c2fc4b4796502a74bec5cd4f5c`. Set `RUST_JS_DIR` if the checkout lives elsewhere. From the Agent Pixel repository root, link the compiler's runtime, build the browser assets and start the loopback server.

~~~sh
bun install --frozen-lockfile
bun setup-rust-js.ts
bun run build
cargo run --locked --manifest-path server/Cargo.toml
~~~

Open `http://127.0.0.1:4173/` for a browser-only draft or `http://127.0.0.1:4173/?server` for the native document service. **Save to server** writes a revision to `data/document.json`. A stale save leaves the local draft intact. **Load server copy** replaces it with the current server revision.

Select a shape on the poster or in Layers. Drag it, edit its bounds in the inspector, or change its fill. A WebMCP-capable browser exposes `read_design` and `edit_layer` while the page is open. To try an agent edit, read the scene, then call `edit_layer` with `{"id":"satellite","fill":"#247a69"}`.

## Rust source and generated JavaScript

- `frontend/src` contains the editor, React components written with `jsx!`, and the WebMCP tool definitions and handlers.
- `core/src` contains the poster, edits, bounds and validation shared by the worker and the native server.
- `site` contains the HTML entry, styles and browser adapters for worker messages, storage, fetch and WebMCP registration.

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
