# Regex GUI
Simple desktop app written in Rust using `eframe`/`egui`. It allows adding
regex rules that map file names to a destination directory. A "Dry Run"
checkbox toggles whether renames should actually happen.

Logs are captured using the [`tracing`](https://crates.io/crates/tracing)
ecosystem and displayed in the UI. The global subscriber filters logs for this
crate at the `INFO` level so that only relevant messages appear.

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

## Web

To run the application in a browser you need
[Trunk](https://trunkrs.dev/) and the `wasm32-unknown-unknown` target:

```
rustup target add wasm32-unknown-unknown
trunk serve
```

Alternatively you can run the web version with Docker:

```
docker compose up web --build
```

Then open <http://localhost:8080> in your browser.

## Tauri (Vite)

To run the application inside a Tauri window using Vite:

```
# start the Vite dev server
npm run dev &
# in another terminal, launch Tauri
npm run tauri
```

The `tauri.conf.json` is configured to load `http://localhost:5173` during development.
