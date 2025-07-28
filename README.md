# Makako HTTP Client

A fast, lightweight, and native desktop HTTP client built in Rust with [GPUI](https://gpui.rs/). Inspired by Bruno — all state and collections live on your local filesystem. No cloud, no accounts.

## Status

> **v0.5 — Async request execution complete**
> Send button fires a real HTTP request in a background thread. The response panel displays status code, latency (ms), and response body. Storage layer is next.

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
    └── mod.rs                    # File-based collection persistence — TODO
```

### Module responsibilities

| Module           | Responsibility                                                        |
|------------------|-----------------------------------------------------------------------|
| `ui_module`      | All GPUI rendering: 3-panel layout, request editor, response viewer   |
| `network_module` | Async HTTP calls via `reqwest` (GET, POST, PUT, DELETE)               |
| `storage_module` | Read/write request collections from local `.bru`-style text files     |

## UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Sidebar (240 px)  │  Request Editor (flex)  │ Response  │
│                    │                         │ (420 px)  │
│  Collections &     │  URL · Method · Headers │           │
│  saved requests    │  Body (JSON)            │  Status   │
│                    │                         │  Time     │
│                    │                         │  Body     │
└──────────────────────────────────────────────────────────┘
```

## Goals (v1)

- [x] 3-panel layout shell
- [x] URL input + HTTP method selector (GET, POST, PUT, DELETE)
- [x] Headers editor (key-value pairs, add/remove rows)
- [x] JSON body textarea (code editor with syntax highlighting)
- [x] Async request execution and response display
- [ ] Save/load requests from local files

## Non-Goals

- No cloud sync or user accounts
- No GraphQL, gRPC, or WebSockets (v1)
- No automatic OAuth2 flows — pass tokens manually in headers

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

| Crate             | Purpose                              |
|-------------------|--------------------------------------|
| `gpui`            | High-performance native UI framework |
| `gpui-component`  | Additional GPUI component utilities  |
| `core-text`       | macOS CoreText bindings              |
| `reqwest`         | HTTP client (blocking, used in background thread) |
| `futures`         | `oneshot` channel to bridge thread → async executor |
