# Rust Path Tracer (WGPU)

A cross-platform, physically based path tracer written in Rust using WGPU and egui. It supports rendering to Native environments (macOS/Windows/Linux) and WebAssembly (via Trunk).

## Features
- **Physically Based Rendering (PBR)**: Uses GGX Microfacet distribution, Schlick's Fresnel approximation, and Smith's Geometry function.
- **Progressive Accumulation**: Seamlessly computes N-bounces across interactive time-steps.
- **Anti-aliasing (MSAA)**: Stochastic Sub-pixel jitter combined with aperture DoF.
- **Live GUI**: Fully dynamic `egui` integration controlling camera settings, materials, ray limits, and rendering statistics.

## Building and Running

### Native (Desktop)
Ensure you have the latest Rust toolchain installed.
```bash
cargo run --release
```

### WebAssembly (GitHub Pages or Local Web)
This project requires [Trunk](https://trunkrs.dev/) and the `wasm32-unknown-unknown` target.
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve web/index.html --release
```
Then navigate to `http://127.0.0.1:8080` in your browser.
