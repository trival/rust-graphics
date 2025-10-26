# Experiments with WGPU and Rust-GPU

This repository contains some experiments with WGPU and Rust-GPU. It utilizes the work-in-progress [trivalibs_painter](https://github.com/trivial-space/trivalibs/tree/main/crates/trivalibs_painter) rendering lib to ease the use of WGPU.

## Shader crates

Shaders are compiled from Rust to SPIR-V using [cargo-gpu](https://github.com/Rust-GPU/cargo-gpu).

To manually compile shaders, run `cargo gpu build` in the shader crate directory:

```bash
cd sketches/your-sketch-name/shader
cargo gpu build
```

Shader crates can also be used from CPU Rust code, just like any other crate.

## Development Workflow

### Hot Reload with run-watch

For the best development experience, use the `dev` command that automatically rebuilds both your application code and shaders:

```bash
# From project root - just provide the sketch name
cargo dev your-sketch-name # sketch directory relative path within /sketches
```

The `dev` command is configured in .cargo/config.toml and uses the run-watch script.

This will:

- Watch `sketches/your-sketch-name/src/` for Rust code changes
- Watch `sketches/your-sketch-name/shader/src/` for shader code changes
- Automatically rebuild and restart the sketch when code changes
- Automatically recompile shaders with `cargo gpu build` when shader files change
- Continue running even if builds fail (keeps last working version running)
- Show all output in a single terminal with `[Main]` and `[Shader]` prefixes

The CanvasApp trait detects shader updates and reloads them at runtime, so you get immediate visual feedback.
