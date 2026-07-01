# posterm

A fast and beautiful TUI REST client, built in Rust.

## Install

### From source (requires Rust)

```bash
cargo install --git https://github.com/igor-gregori/posterm.git
```

### Pre-compiled binaries

Download from [GitHub Releases](https://github.com/igor-gregori/posterm/releases/latest):

- **Linux** — `posterm-x86_64-unknown-linux-gnu.tar.gz`
- **macOS (Intel)** — `posterm-x86_64-apple-darwin.tar.gz`
- **macOS (Apple Silicon)** — `posterm-aarch64-apple-darwin.tar.gz`
- **Windows** — `posterm-x86_64-pc-windows-msvc.zip`

Extract and move to a directory in your PATH.

## Features

- **Request builder** — GET, POST, PUT, DELETE, PATCH with headers, body, and query params
- **Response viewer** — Status, headers, and body with syntax highlighting
- **Collections** — Save and organize requests in folders
- **Environments** — Profiles (dev, staging, prod) with `{{variable}}` interpolation

## Keybindings

Press `F1` in-app for the full list. Key shortcuts:

| Key | Action |
|-----|--------|
| `Ctrl+R` | Send request |
| `Ctrl+T` | Cycle method |
| `Ctrl+U` | Edit URL |
| `Ctrl+H` | Edit headers |
| `Ctrl+B` | Edit body |
| `Ctrl+P` | Edit params |
| `Ctrl+S` | Save to collection |
| `Ctrl+E` | Switch environment |
| `Ctrl+W` | Edit env variables |
| `Tab` | Switch panel |
| `F1` | Help |
| `q` | Quit |

## Tech Stack

| Layer | Crate |
|-------|-------|
| TUI | ratatui + crossterm |
| HTTP | reqwest (rustls) |
| Async | tokio |
| Serialization | serde + serde_json |
| Highlighting | syntect |

## License

MIT
