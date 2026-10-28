# Testing record

## Executed in the generation environment
- Project files and ZIP structure were created.
- Archive integrity and required project paths were checked by the packaging script.

## Not executed
- `cargo check`, `cargo test`, and graphical launch were **not run** in this environment. Dependency fetching/building was unavailable here.
- Windows, Linux, and macOS builds were not run.
- Runtime execution, file picker behavior, and stop behavior require verification on a machine with Rust and the relevant runtimes installed.

## Reproducible verification
Run:

```sh
cargo check
cargo test
cargo run
```

Then create a Python file containing `print("VYRON works")`, open it, click Run, and confirm the output console displays the real output. Test Stop using a long-running trusted script. Try a missing runtime by configuring an invalid executable path in a future runtime-settings implementation; in this release, missing executable errors are displayed in the console.
