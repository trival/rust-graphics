# Experiments with WGPU and Rust-GPU

This repository contains some experiments with WGPU and Rust-GPU. It utilizes the work-in-progress [trivalibs_painter](https://github.com/trivial-space/trivalibs/tree/main/crates/trivalibs_painter) rendering lib to ease the use of WGPU.

## Shader crates

Shaders are compiled from Rust to rspirv shaders using [cargo-gpu](https://github.com/Rust-GPU/cargo-gpu).

To compile a shader crate, run `cargo gpu build` in the crate directory.

To watch compile, install `watchexec` and run `watchexec -r -e rs cargo gpu build` in the shader crate directory.
The CanvasApp trait detects updates in the shader files and reloads them at runtime.

These crates can also be used from CPU Rust code, just like any other crate.

### Hot Reload Development with run-watch

For faster iteration during development, use the `run-watch` utility that automatically rebuilds and restarts your sketch when source files change:

```bash
# From project root
cargo run -p run-watch -- sketches/your-sketch-name
```

This will:

- Watch the `src/` directory of your sketch for changes
- Automatically rebuild when files are modified
- Restart the sketch process after successful builds
- Continue running even if builds fail (keeps last working version running)
