# Makako HTTP Client

A fast, lightweight, and native desktop HTTP client built in Rust with [GPUI](https://gpui.rs/). Inspired by Bruno — all state and collections live on your local filesystem. No cloud, no accounts.

## Status

> **v2.0 — In progress: Collections & Environments**
> v1 complete. Now building a real file-explorer sidebar, hierarchical collection tree, and `{{variable}}` interpolation via `env.json`.

## Architecture

```
src/
├── main.rs                       # Entry point — opens the window and wires modules
├── ui_module/
│   ├── mod.rs                    # AppView: 3-panel shell, request bar, method selector
│   ├── headers_editor.rs         # HeadersEditor sub-view (key-value pairs)
│   └── response_panel.rs         # ResponsePanel sub-view (status, latency, body)
├── network_module/
│   └── mod.rs                    # HTTP execution (reqwest blocking + oneshot channel)
└── storage_module/
    └── mod.rs                    # JSON persistence in ~/Documents/Makako/default/
```

### Module responsibilities

| Module           | Responsibility                                                        |
|------------------|-----------------------------------------------------------------------|
| `ui_module`      | All GPUI rendering: 3-panel layout, request editor, response viewer   |
| `network_module` | Async HTTP calls via `reqwest` (GET, POST, PUT, DELETE)               |
| `storage_module` | Read/write request collections and env files from local filesystem    |

## UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Sidebar (240 px)  │  Request Editor (flex)  │ Response  │
│                    │                         │ (420 px)  │
│  📁 collection/    │  URL · Method · Headers │           │
│    📄 get-users    │  Body (JSON)            │  Status   │
│    📄 post-login   │                         │  Time     │
│  📁 auth/          │  {{base_url}}/endpoint  │  Body     │
│    📄 refresh      │                         │           │
└──────────────────────────────────────────────────────────┘
```

## Goals (v2) — In Progress

- [x] Sidebar file-explorer: read `~/Documents/Makako/` as a directory tree
- [x] Tree node model: `CollectionNode::Folder` / `CollectionNode::Request`
- [x] Collapsible folder nodes with indented children
- [x] Click on a `.json` file → load request into editor
- [ ] Environment variables: `env.json` in collection root
- [ ] `{{variable}}` interpolation applied to URL and headers before sending

## Non-Goals

- No cloud sync or user accounts
- No GraphQL, gRPC, or WebSockets (v1/v2)
- No automatic OAuth2 flows — pass tokens manually in headers
- No multiple open tabs (v2) — one active request at a time
- No `.bru` format parsing — JSON collections for now

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)
- macOS (GPUI is optimized for macOS; requires `core-text`)

## Running

```bash
cargo run
```

## Building a release binary

```bash
cargo build --release
```

## Dependencies

| Crate                  | Purpose                                              |
|------------------------|------------------------------------------------------|
| `gpui`                 | High-performance native UI framework                 |
| `gpui-component`       | Additional GPUI component utilities                  |
| `core-text`            | macOS CoreText bindings                              |
| `reqwest`              | HTTP client (blocking, used in background thread)    |
| `futures`              | `oneshot` channel to bridge thread → async executor  |
| `serde` / `serde_json` | Serialize/deserialize saved requests as JSON         |
| `dirs`                 | Resolve `~/Documents/` cross-platform                |
