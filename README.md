# Youtube Server

This project takes Piped and makes using it as simple as running 1 binary. That's it! So portable and easy to use.

## Cli options
`--ip` - The ip to listen on in ipv4 or ipv6 format (default `0.0.0.0`)

`--port` - The port to listen on (default `8080`)

## Building

You need node and `pnpm` installed first. You also need [Rust installed](https://rustup.rs/)
1. Run `pnpm -C piped install`
2. Run `pnpm -C piped build`
3. Run `cargo build --release`

Note: There's `build-piped` scripts for windows/linux to execute the first 2 commands automatically
