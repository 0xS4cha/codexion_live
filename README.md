# Codexion - Live

Codexion Live Bridge tool (Rust)


## Description

This project is a Rust tool built specifically to assist the **Live Mode** of the [Codexion Visualizer](https://github.com/0xS4cha/codexion_visualizer). It has been created as part of the 42 curriculum (Codexion - Live Visualizer - 42 CC) to act as a bridge: reading real-time data from standard input and broadcasting it via WebSockets, allowing the visualizer to render live updates seamlessly.

## Instruction

The Codexion Live Bridge can be built locally with a standard Rust (Cargo) toolchain or using the provided Makefile.

```bash
make
```

Then run it:

```bash
make run
```

## Usage

### Local Development

```bash
make debug
make run
# or simply
make cargo-run
```

### Build

```bash
make release
```

### Formatting & Linting

```bash
make fmt
make lint
```

### Data Input

Depending on your data source, you can pipe data directly into the application:
- read from a live log file: `tail -f logs.txt | ./codexion_live`
- run another program and pipe output: `./my_program | ./codexion_live`

> The application listens on `ws://127.0.0.1:8080` by default. You can change the port using the `PORT` environment variable.

## Features

The live bridge layer is designed to provide:
- **Real-time broadcasting** (WebSocket server using tokio-tungstenite)
- **Multi-client support** (broadcasts incoming lines to all connected clients)
- **History buffering** (caches previous lines and sends them upon new connections)
- **Simple integration** (reads directly from stdin line by line)





## Feedback

If you have feedback, open an issue or contact the author.
