# epoll-notify-rust

This repository is a small Rust experiment for exploring Linux epoll and non-blocking TCP sockets. It is meant as a learning project for observing how an event-driven server handles many connections rather than as a production-ready messaging system.

## What this project does

The code shows a basic server/client setup built around:

- Linux epoll for waiting on many sockets
- non-blocking TCP connections
- a simple event loop for incoming activity
- low-level libc bindings for networking primitives

## Current scope

The project is intentionally narrow. It focuses on the mechanics of connection handling and epoll-driven event processing.

## Project layout

- src/epoll.rs: helpers for epoll registration and event creation
- src/bin/server.rs: a simple TCP server using epoll
- src/bin/client.rs: a basic client that opens connections to the server

## Build

From the repository root, run:

```bash
cargo build
```

## Run

Start the server:

```bash
cargo run --bin server
```

In another terminal, start the client:

```bash
cargo run --bin client
```

## Notes

- This project is Linux-specific and relies on epoll.
- It binds to port 8080, so that port should be available.
- The code is experimental and intended for learning and experimentation.
