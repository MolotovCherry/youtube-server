# Youtube Server

This project takes [piped](https://github.com/TeamPiped/Piped) and makes it easy to run. Click on the binary to run. That's it! So portable and easy to use, even your grandmother could do it.

## Cli options
`--ip` - The ip to listen on in ipv4 or ipv6 format (default `0.0.0.0`)

`--port` - The port to listen on (default `8080`)

## Building

You need node and `pnpm` installed first (and in your PATH). You also need [Rust installed](https://rustup.rs/)
- Run `cargo build --release`
