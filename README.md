# Agent Pixel

A vector design editor for people and WebMCP agents, built with rust-js.

[Open the editor](https://rust-js-lang.github.io/agent-pixel/) · [Read the design](DESIGN.md)

## Run locally

From the repository root, build the browser assets and start the loopback Rust server.

~~~sh
bun run build
cargo run --locked --manifest-path server/Cargo.toml
~~~

Open `http://127.0.0.1:4173/` for a browser-only draft or `http://127.0.0.1:4173/?server` for the native document service. **Save to server** writes a revision to `data/document.json`. A stale save leaves the local draft intact. **Load server copy** replaces it with the current server revision.

Select a shape on the poster or in Layers. Drag it, edit its bounds in the inspector, or change its fill. A WebMCP-capable browser exposes `read_design` and `edit_layer` while the page is open. To try an agent edit, read the scene, then call `edit_layer` with `{"id":"satellite","fill":"#247a69"}`.

## Build and verify

`bun test` exercises the generated browser geometry. `cargo test --locked --manifest-path core/Cargo.toml` exercises native Rust. GitHub Actions runs both and publishes the Bun build of the checked-in modules.

The Rust source of the shared geometry lives in `core/src/lib.rs`. To regenerate its adjacent ES module, use the [rust-js checkout](https://github.com/rust-js-lang/rust-js) at `ae378c0fc55459c2fc4b4796502a74bec5cd4f5c` with its documented compiler setup. Set `RUST_JS_DIR` to that checkout's absolute path.

~~~sh
(cd "$RUST_JS_DIR" && bun install && bun run build:compiler)
mkdir -p core/node_modules/@rust-js
ln -sfn "$RUST_JS_DIR/std" core/node_modules/@rust-js/std
(cd core && RUST_JS_COMPILER="$RUST_JS_DIR/target/debug/rust-js" bun "$RUST_JS_DIR/tooling/cli.js" build)
~~~
