# Usage

The application displays renaming rules in a small data table. Each row lets you
enter a regular expression and the destination path. Dedicated buttons allow you
to add or remove rows and the input fields have ample width for comfortable
typing.

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
