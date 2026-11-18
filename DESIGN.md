# Agent Pixel

Agent Pixel is a vector design editor for a person and an agent working on the same artboard. A designer can place a shape and ask an agent to align it. The agent reads its actual bounds, submits an edit by layer ID, and the designer sees the result immediately.

## The first slice

Ship a single editable poster on GitHub Pages. Its editable geometry lives in artboard coordinates, so the same position means the same thing on a phone, a desktop, and in a tool call. A designer can select, add, move, resize and recolor shapes. A dedicated worker handles document edits. Reloading restores the current browser's work. The page and worker load from relative URLs under `/agent-pixel/`.

## One document, two ways to edit

A versioned snapshot contains an ordered list of layers. Each has a stable ID, shape kind, integer x/y origin, width, height and fill. The last layer paints on top. The first release uses a fixed artboard, rectangles and ellipses. The printed poster caption is a fixed template element. Selection and transient pointer feedback belong to the view.

~~~text
Person       Editor            Worker              Storage
  | drag       |                 |                    |
  |----------->| edit layer      |                    |
Agent          |---------------->| validate geometry  |
  | WebMCP     |                 | and commit         |
  |----------->|---------------->|                    |
  |            |<----------------| resulting snapshot |
  |            | paint + persist |                    |
  |            |------------------------------------->|
  |<-----------| tool result     |                    |
~~~

The worker serializes mutations in arrival order. Validate identity, geometry and fill before changing the scene. Reply to each request with its own result or error. Keep the last successful state on a rejected edit. UI and WebMCP use the same commands. Treat layer values as data and expose only the current document.

## Agent workflow

Expose a read tool to discover artboard dimensions, layer IDs and bounds. A write tool creates a layer or patches fields on an existing ID. For example, read the scene, take the ID of its ellipse and set its horizontal origin to half the difference between the artboard width and the ellipse width. Return the committed scene so an agent can verify placement. Feature-detect browser WebMCP registration and keep the editor usable across browser support levels. [WebMCP imperative API](https://developer.chrome.com/docs/ai/webmcp/imperative-api)

## Rust across runtimes

Keep geometry rules in a Rust library. Compile that library to ES modules with rust-js for the browser worker and link the same crate natively for the optional server. The browser adapter owns DOM, worker messages and WebMCP registration. Compare bounds cases in native Rust and emitted JavaScript before publishing.

In server mode, serve the same static client and add document storage behind a same-origin API. A planned GET returns a snapshot and revision. PUT supplies the base revision. The server validates the complete candidate with the shared rules, commits it atomically and returns the new revision. On conflict preserve the user's local work and offer to reload the server copy. Start with single-user local hosting. Add authentication and document access checks before enabling shared remote editing.

## Performance and growth

For the small poster, copy a committed snapshot across the worker boundary once per edit. Keep pointer motion as a local preview until release. Measure input-to-paint and worker round trips on a 1,000-layer fixture before choosing incremental scene patches or offscreen rendering. A worker can free the UI thread while copying large scenes can still stall it.

## Delivery gates

1. **Pages editor.** Open the project URL in a fresh browser, edit a shape, reload, and recover it. Invoke a registered tool, inspect its result and try an invalid edit. Rebuild the deployed page from a clean checkout.
2. **Native document service.** Save a browser-created document, restart the service, load it again and reject a stale concurrent save. Run the same geometry fixtures in both targets.
3. **Broader design workflow.** Add editable typography, grouped layers, undo transactions and export. Use real agent layout tasks to decide the order. Multi-user collaboration needs a separate conflict model.
