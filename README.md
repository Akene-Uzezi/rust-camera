# rustcamera

A tiny Rust program that queries a V4L2 video device on Linux and prints its driver and card names.

## What it does

`rustcamera` opens `/dev/video0`, asks the Linux kernel for the device's capability information via `ioctl`, and prints:

- Driver name
- Card name

## Building

You need Rust and Cargo installed.

```bash
cargo build
```

## Running

```bash
./target/debug/rustcamera
```

## Requirements

- Linux with V4L2 support
- A video device at `/dev/video0` (a webcam or capture card)
- Permission to read and write `/dev/video0` (you may need to be in the `video` group or run as root)

## Code overview

See `EXPLAINER.md` for a walkthrough of the source code aimed at Go developers.
