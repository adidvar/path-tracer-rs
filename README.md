# Rust Path Tracer 

A small path tracing demo using Rust, wgpu and egui.
It can run on desktop (macOS / Windows / Linux) and in the browser via WebAssembly.

Prerequisites
- Rust toolchain (stable)
- For web: add the wasm target and install Trunk
  - `rustup target add wasm32-unknown-unknown`
  - `cargo install trunk`

Running (Desktop)
- Build and run with Cargo:
  - `cargo run` (debug)
  - `cargo run --release` (optimized)

Running (Web)
- Use Trunk to build and serve:
  - `trunk serve web/index.html --release`
- Open the address printed by Trunk in a modern browser.

Notes
- If GPU initialization fails, check your system drivers (desktop) or WebGPU support in your browser (web).
- Shaders and assets live in `assets/` and must be available to the build tool you use.
