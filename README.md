# VYRON IDE 0.1.0

VYRON is a native Rust desktop code-editor foundation with a graphite interface. Its UI is rendered by `egui/eframe`; it does **not** use HTML, CSS, JavaScript, React, or a webview for its own interface.

## Implemented in this release

- Native cross-platform desktop shell (Windows, Linux, macOS; build availability depends on toolchain/system libraries).
- Open a folder/workspace, browse nested files, open files in tabs, edit text, save/save-as.
- File creation, folder creation, rename and delete actions from the explorer.
- File-manager upload/import: use **Import File** to pick a file from your computer/phone file manager and copy it into the workspace.
- Multi-tab editing, dirty indicators, line numbers, syntax-aware coloring for common languages via Syntect, find-in-current-file, go-to-line, word wrap toggle, font-size controls.
- Symbol outline for common Rust/Python/JS/TS/Go/C/C++/Java source patterns. It is a lightweight outline, not a full Tree-sitter/LSP semantic hierarchy yet.
- Command palette (`Ctrl+Shift+P` / `Cmd+Shift+P`), theme selection (Graphite Dark, Midnight, Light), persistent preferences.
- Run current file using detected Python, Node.js, or Rust/Cargo commands; real stdout/stderr and exit status in the output console; stop current process.
- Workspace-wide text search.
- App icon asset (`assets/vyron.svg`) and favicon-style brand asset (`assets/favicon.svg`).

## Honest limitations

This is an early foundation, not full VS Code parity. The bottom panel is a process output console, **not** an interactive PTY terminal. Interactive programs requiring keyboard input are not supported in this console. Debug Adapter Protocol, breakpoints, LSP servers, full Tree-sitter parsing, multi-cursor editing, minimap, integrated Git staging/commit UI, and extension installation are not implemented in 0.1.0. The outline is best-effort and does not claim full semantic parsing. Run tasks execute commands from the opened workspace; only run projects you trust.

The UI uses a single-line process runner for this release. Stop terminates the direct child process; programs that spawn detached grandchildren may require platform-specific process-tree management.

## Requirements

- Rust stable toolchain and Cargo.
- Python 3 for Python files, Node.js for JavaScript/TypeScript, and Cargo for Rust projects, if you want to run those languages.
- Linux: depending on distribution, install the GTK/WebKit-related system packages required by `rfd`/`eframe`'s native window backend. No browser/webview is used for the application UI.

## Build and run

```sh
cargo run
```

Release build:

```sh
cargo build --release
```

The executable is created under `target/release/`.

## Use

1. Launch VYRON.
2. Choose **Open Folder** to select a workspace, or **New Workspace** to create one.
3. Open a file from Explorer. Edit and press `Ctrl+S` (macOS: `Cmd+S`) to save.
4. Use **Import File** to choose a file from the device file picker and copy it into the workspace.
5. Press **Run** to execute the current supported file, or use the command palette.
6. Check process output and errors in the bottom console.

## GitHub and deployment

This is a **desktop Rust application**, not a GitHub Pages website. GitHub Pages cannot run a native Rust desktop executable in the browser. To put the source on GitHub, create a repository named `VYRON-IDE`, upload the contents of this folder, and push to the default branch. The included CI workflow runs `cargo check` and tests on three operating systems. Releases can be built with the included commands or a release workflow added later.

For phone-only repository setup, use GitHub's web interface: create a new repository, then upload the project files using **Add file → Upload files**. Note that GitHub's web uploader is not a compiler; a successful upload does not mean the app has been built.

## Security

Running code executes programs on your machine with your user permissions. Do not run unfamiliar or untrusted projects. VYRON does not automatically run a project when it is opened.
