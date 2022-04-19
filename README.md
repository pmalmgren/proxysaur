# proxysaur

A network debugging proxy multitool powered by WebAssembly.

## Installation

### macOS

```bash
$ brew tap proxysaur/proxysaur
$ brew install proxysaur
```

### Debian-based Linux

```bash
$ apt-get install proxysaur
```

### Releases page

Navigate to the [releases](releases) page and download a compiled release for your platform.

## Getting started

### Configuration

proxysaur looks for a configuration file called `proxysaur.toml` in the following directories:

1. The directory specified in the CLI flag `--config-path`
2. The current directory
3. In `~/.config/proxysaur/`

### Server types

proxysaur can proxy and debug a few different kinds of servers:

- HTTP
- Redis
- gRPC
- PostgreSQL