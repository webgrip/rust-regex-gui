# Regex GUI

Simple desktop app written in Rust using `eframe`/`egui`. It allows adding
regex rules that map file names to a destination directory. A "Dry Run"
checkbox toggles whether renames should actually happen.

Logs are captured using the [`tracing`](https://crates.io/crates/tracing)
ecosystem and displayed in the UI.

## Running locally

```
cargo run
```

## Docker

Build and run the app with Docker:

```
docker compose up --build
```

The container uses X11 forwarding, so an X11 server must be available on the
host.
