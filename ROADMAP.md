# Agent Pixel production roadmap

## Readiness

**Agent Pixel is ready for prototyping and evaluation. Use a separate system of record for production designs today.** The editor can create and persist browser drafts, edit shapes and text, place linked Canopy UI variants, and let a person and a WebMCP agent work on the same artboard. The reminder demo shows a clickable journey and a developer handoff. It uses predefined screens and interactions.

The release blockers are recoverable project storage, dependable editing of real multi-screen files, reusable components authored by designers, and a handoff another person can open and inspect. The native Rust service stores a single document over loopback. It demonstrates revision checks and atomic replacement, while the dashboard's project catalog and drafts live in browser storage. The scene is a flat, ordered layer list with a fixed 480 × 600 artboard and a 1,000-layer limit per scene. Canopy is the built-in component library.

### M1 checkpoint

The dashboard and WebMCP can export and restore a versioned project bundle with the current artboards, prototype links, and a pinned component library. The editor supports attributed undo and redo within an open session. Agents can submit an atomic batch at a known revision and undo or redo the entire batch. Draft saves now use IndexedDB revisions and retain the previous committed snapshot. Opening a legacy draft migrates a validated copy, and a concurrent edit creates a durable recovery project. An interrupted write can be retried from the current editor. People and agents can copy the previous saved snapshot into a separate project, with the recorded component library when available. The catalog and imported bundles now use IndexedDB and are committed together. Legacy localStorage projects keep their IDs and original records during migration. After a failed write, people and agents can export the in-memory screen with the rest of the project and restore it after closing the editor. That path requires exporting before leaving. PNG, JPEG, and WebP image layers reference assets stored atomically with each artboard revision. People can upload a file and agents can place an image within a reversible WebMCP batch. Bundles validate image references and carry the bytes into a fresh browser. Font handling, storage limits for larger projects, and the full M1 exit test remain open.

## Target workflow

A small product team can design a feature from a blank project, using a human and a coding agent together:

~~~text
Create a project → lay out screens → reuse a library → wire the flow
    → review a frozen revision → hand off measurements and assets → revise
~~~

The first production target is solo authoring with portable, versioned handoff. A team pilot adds private shared projects and reviewer access. Each milestone has an exit test that someone can run on a new project.

## M1 · Protect work

Ship this first so larger designs are safe to edit.

- **Versioned project format.** Define a project as a catalog of pages or screens with stable IDs, scene revisions, and asset and component references. Migrate existing drafts and keep the old keys until the imported project reopens successfully. Validate migrations in the shared Rust model.
- **Durable local saves.** Store projects and assets in IndexedDB. Atomically commit a revision with its assets, show saving/saved/failed status, and keep a recoverable checkpoint. Give each open tab a revision and surface a conflict if both edit the same project.
- **Portable backup.** Export a complete project bundle and import it into a fresh browser. Include assets and either the referenced component revisions or a snapshot required to render them. Validate the entire bundle before publishing it into the catalog.
- **Undo and redo.** Record one transaction per user gesture or tool batch. Support undo for deletes and agent edits, with a history entry identifying the action. Ensure a failed command leaves both the scene and its history intact.

**Exit test.** Migrate a current draft with linked components, simulate interruption and storage failure, and recover the last committed revision. Export the recovered design and reopen it in a fresh profile with the same artboards, fonts or declared fallbacks, assets, and references. Undo and redo a multi-layer agent edit as one operation.

## M2 · Design a feature from scratch

Build the editing primitives in this order so frames, constraints, and component instances share the same document model. The editor now keeps canvas scrolling inside the workspace. The Layers panel supports persistent nested groups and linked component children, with the same hierarchy edits available through WebMCP. Named screens, independently sized frames, and the other editing primitives remain ahead.

1. **Screens and layers.** Create, name, duplicate, reorder, and delete pages and frames of arbitrary supported sizes. Add nested frames, multi-select, groups, layer order, visibility, locks, clipping, and copy/paste. Give each screen a stable route.
2. **Precision editing.** Add resize handles, alignment, distribution, snapping, and keyboard nudge. Support fill, stroke, corner radius, opacity, basic vector paths, and image placement. Add usable type controls for family, weight, line height, alignment, and wrapping. Define how missing fonts are reported and rendered.
3. **Responsive layout.** Implement row and column layout with gap, padding, alignment, fixed and content-sized dimensions, and fill behavior. Add pinned constraints for absolute children. Let designers resize a frame and inspect the result at mobile and desktop widths.
4. **Team-authored libraries.** Create a component from selected layers and define its variant properties. Reuse components across projects. Support instance movement, text and image overrides, variant switching, and detach. Track the source revision, preview a library update, and allow a rollback without losing supported overrides.

**Exit test.** Build a six-screen feature without starting from a demo. Reuse at least three linked components across mobile and desktop frames. Change the library, pick up the intended updates, switch a variant, and resize a frame without having to reposition each child by hand.

## M3 · Review and hand off

- **Authorable prototypes.** Let designers choose a starting screen and connect clicks to frames in the editor. Support back, a modal or overlay, and a simple failure/retry branch. Provide a keyboard-navigable preview that reads the selected revision.
- **Inspect a frozen revision.** Give developers a URL or portable snapshot they can open without the designer's local draft. Expose measurements, spacing, typography, colors, tokens, component properties, and the interaction map. Link annotations and states to stable screen or layer IDs. Keep the handoff revision fixed while the author continues editing.
- **Exports and review.** Ship PNG for screen review and SVG for supported vector content. Validate exported dimensions and disclose unsupported attributes or font substitutions. Include a screen list with annotations and an acceptance checklist. Start with snapshot review and add comments anchored to a revision in the team pilot.
- **Acceptance with a developer.** Use a real feature with empty, loading, error, and success states. Ask a second person to implement one feature from the handoff. Treat questions that the file should answer as product defects.

**Exit test.** A reviewer opens the prototype from a fresh browser. A developer can find the states, inspect values, download assets, and implement the agreed screen without needing the author to provide missing design details.

## M4 · Pilot with a real team

- Extend the optional Rust service to store multiple projects, versions, libraries, and assets. Add authentication and project-level owner, editor, and viewer roles before hosting private work. Enforce permissions on the server for both browser and agent calls. Add private share links and a documented conflict flow that preserves both local and remote work.
- Ship operational basics for the chosen host. Exercise backup and restore, version upgrade and rollback, and recovery after interrupted uploads. Document how to run the service privately, who can access a design, and how to export or remove a project.
- Run a real feature cycle with a small pilot team from design through implementation. Use revision-based saves and asynchronous review for this stage.

**Exit test.** A pilot team completes one real feature cycle, including library revision, review, and handoff. A viewer cannot write or discover other private projects. The team can restore a backup and recover a conflicting edit.

## Agent and release gates throughout

- **One command model.** Put validation, transactions, and migrations in the shared Rust document core. UI interactions and WebMCP tools use the same rules. Keep rendering and pointer previews outside committed state.
- **Agent workflow parity.** As editor capabilities land, expose project and frame discovery, scoped reads, and atomic edit batches for operations agents need. Return stable IDs, revisions, and actionable errors. Reject stale writes. Attribute agent changes and make them reversible. Keep authoring usable on browsers without WebMCP.
- **Prove behavior.** Run native Rust and compiled JavaScript against the same fixtures for migrations, layout, and edits. Check that the pinned rust-js compiler reproduces the checked-in output. Use Playwright for blank-project authoring, agent batches, undo, link updates, backup/import, and a fresh-browser handoff.
- **Measure before raising limits.** Build representative fixtures with long text, images, linked instances, and many frames. Start with a six-screen project and one 1,000-layer stress frame. Record pointer-to-paint, worker round trips, startup, and export on a named reference machine. Set budgets from the measurements and gate releases on agreed regressions.
- **Validate the supported environment.** Start the pilot with desktop Chromium and the WebMCP test configuration. Exercise keyboard navigation, focus visibility, mobile preview, and degraded storage or browser capability. Publish the supported browser and file-size ranges after testing them.

Defer real-time multiplayer, full Figma import, plugin hosting, advanced illustration tools, and animation timelines. The next build starts with the project bundle and recovery path, then adds reversible transactions and general-purpose frames.
