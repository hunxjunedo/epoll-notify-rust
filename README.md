# epoll-notify-rust

A small Linux-only Rust experiment for stress-testing **epoll with many concurrent TCP connections**.

TCP connect/accept and a tiny auth handshake exist as scaffolding so sockets stay open and identifiable. The point of the project is observing an epoll event loop under many FDs, not building a messaging product.

## What you get

| Binary | Role |
|--------|------|
| `server` | Non-blocking listen socket + epoll loop: accept, authenticate, track open FDs |
| `client` | Opens many non-blocking TCP connections, authenticates each, then holds them open |

Shared helpers:

- `src/epoll.rs` — thin wrappers around `epoll_ctl` / event structs
- `src/protocol.rs` — minimal framing for `AUTH` / `TEXT` / `CLOSE`

## Requirements

- Linux (uses `epoll`, `accept4`, etc.)
- Rust toolchain (`cargo`)
- Enough file descriptors for your target connection count (`ulimit -n`)

## Build

```bash
cargo build
```

## Run

Terminal 1 — start the server:

```bash
cargo run --bin server
```

Terminal 2 — open many connections (currently hard-coded to 1000):

```bash
cargo run --bin client
```

Both processes keep running. The client parks after connects + auth so the sockets stay open. Stop either side with **Ctrl-C**.

## Layout

```
src/
  epoll.rs          epoll helpers
  protocol.rs       AUTH / TEXT / CLOSE framing
  bin/server.rs     epoll TCP server
  bin/client.rs     many-connection client
```

## Notes

- Linux-specific; binds to port 8080.
- Auth is minimal: client id + dummy password; server stores `fd → id` in `open_connections`.
- A future step is a client thread pool to drive traffic across the held connections.
- This is a learning/experiment repo, not a production service.
