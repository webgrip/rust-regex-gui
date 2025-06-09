# Usage

## Running locally

```bash
cargo run
```

## Docker

Build and run the application using Docker:

```bash
docker compose up --build
```

An X11 server must be available on the host as the container forwards the UI.

## Web

To build and run in a browser:

```bash
rustup target add wasm32-unknown-unknown
trunk serve
```

Or use Docker:

```bash
docker compose up web --build
```

Then open <http://localhost:8080> in your browser.
