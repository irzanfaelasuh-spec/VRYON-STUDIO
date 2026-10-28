# Architecture

- `src/main.rs`: native app entry point.
- `src/app.rs`: application state, frame lifecycle, shortcuts, themes and command palette.
- `src/workspace.rs`: filesystem workspace operations and file discovery.
- `src/editor.rs`: tab state, text editing, syntax coloring, file saving and symbol outline.
- `src/execution.rs`: runtime detection, process execution and cancellation.
- `src/settings.rs`: persistent settings.

UI is rendered with egui/eframe. Filesystem work and process output are handled by dedicated modules; the process runner streams output from a background thread and returns status over a channel. No frontend web technologies are used.
