# epoll-notify-rust

A small Linux-only Rust experiment for stress-testing **epoll with many concurrent TCP connections**.

TCP connect/accept and a minimal AUTH handshake identify sockets. TEXT/CLOSE messaging in `protocol.rs` is not driven yet — next step is a client thread pool for traffic.

## What you get

| Binary | Role |
|--------|------|
| `server` | Non-blocking listen socket + epoll loop: accept, AUTH into `open_connections` |
| `client` | Opens many non-blocking TCP connections, sends AUTH, then holds them open |

Shared helpers:

- `src/epoll.rs` — thin wrappers around `epoll_ctl` / event structs
- `src/protocol.rs` — AUTH (used) / TEXT / CLOSE (deferred until thread pool)

## Requirements

- Linux (uses [`epoll`](https://man7.org/linux/man-pages/man7/epoll.7.html), [`accept4`](https://man7.org/linux/man-pages/man2/accept.2.html), etc.)
- [Rust](https://www.rust-lang.org/tools/install) toolchain (`cargo`)
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

Terminal 2 — open connections (pass the count as a CLI argument):

```bash
cargo run --bin client -- 1000
```

Examples:

```bash
cargo run --bin client -- 100      # small smoke test
cargo run --bin client -- 5000     # heavier run (check ulimit -n first)
```

The client requires `<n_connections>`; there is no default.

Both processes keep running. The client parks after connects + AUTH so the sockets stay open. Stop either side with **Ctrl-C**.

Optional: inspect held connections while both processes are up:

```bash
ss -tn state established '( dport = :8080 or sport = :8080 )' | wc -l
```

## Layout

```
src/
  epoll.rs          epoll helpers
  protocol.rs       AUTH (used) / TEXT / CLOSE (deferred)
  bin/server.rs     epoll TCP server
  bin/client.rs     many-connection client
```

## Cautions

- **Linux only.** macOS/BSD/Windows do not provide `epoll`; this project will not build or run there as written.
- **Port 8080.** The server binds `0.0.0.0:8080`. Stop any other process using that port first, or change the port in code.
- **File descriptor limits.** Each connection consumes an FD on both client and server. At high counts you will hit `ulimit -n` / `/proc/sys/fs/file-max` long before epoll itself fails. Raise the soft limit if needed: `ulimit -n 65535`.
- **Ephemeral ports.** Opening thousands of client connections to one local port can exhaust the ephemeral port range or hit `somaxconn` / backlog limits. Failures often look like `ECONNREFUSED` or stuck connects, not an epoll bug.
- **No security model.** AUTH is a dummy id/password handshake for bookkeeping only. Do not expose this on a public network.
- **Not production software.** Logging is verbose, error handling is experimental, and TEXT/CLOSE are unfinished. Treat this as a learning / load-experiment harness.
- **Ctrl-C closes everything.** When the client process exits, the kernel closes all its sockets (FINs). That is expected; leave the client running to keep connections open.

## References

- [epoll(7)](https://man7.org/linux/man-pages/man7/epoll.7.html) — event polling overview
- [epoll_ctl(2)](https://man7.org/linux/man-pages/man2/epoll_ctl.2.html) / [epoll_wait(2)](https://man7.org/linux/man-pages/man2/epoll_wait.2.html)
- [socket(7)](https://man7.org/linux/man-pages/man7/socket.7.html) — `SO_ERROR`, non-blocking connect notes
- [accept(2)](https://man7.org/linux/man-pages/man2/accept.2.html) — `accept4` / `SOCK_NONBLOCK`
- [libc crate](https://docs.rs/libc) — Rust FFI bindings used here
- [ss(8)](https://man7.org/linux/man-pages/man8/ss.8.html) — inspect socket state while testing

## Notes

- AUTH is minimal (id + dummy password); server moves `fd → id` into `open_connections`.
- TEXT/CLOSE traffic is deferred for the upcoming client thread pool.
