# posterm

A fast and beautiful TUI REST client, built in Rust.

## Features

- **Request builder** — GET, POST, PUT, DELETE, PATCH with headers, body, and query params
- **Response viewer** — Status, headers, and body with syntax highlighting
- **Collections** — Save and organize requests in folders
- **History** — Recent requests log
- **Environment variables** — Profiles (dev, staging, prod) with `{{variable}}` interpolation

## Tech Stack

| Layer | Crate |
|-------|-------|
| TUI | ratatui + crossterm |
| HTTP | reqwest (rustls) |
| Async | tokio |
| Serialization | serde + serde_json |
| Highlighting | syntect |

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Send request |
| `Tab` | Navigate panels |
| `Ctrl+S` | Save request to collection |
| `Ctrl+E` | Switch environment |
| `q` / `Ctrl+C` | Quit |

## Building

```bash
cargo build --release
```

## License

MIT
