# stimulus

A modular tool for generating traffic and L7 attacks. Written in Rust.

## Features

- High performance
- Multiple protocols and methods
- Extenible - users can easily add their own methods.
- Traffic counters and measurements

## Usage

```
Usage: stimulus [OPTIONS] <TARGET>

Arguments:
  <TARGET>  IP or Domain of the target server. You can also use port here with ":"

Options:
  -w, --workers <WORKERS>    Number of workers. [default: 100]
  -d, --duration <DURATION>  Attack duration. Available formats: seconds, minutes, hours [default: 1m]
  -m, --method <METHOD>      Attack method. Available methods: join, ping, icmp [default: ping]
  -h, --help                 Print help
```

## Building

```bash
cargo build --release
```
