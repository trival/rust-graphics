# Experiments with WGPU and Rust-GPU

This repository contains some experiments with WGPU and Rust-GPU. It utilizes the work-in-progress [trivalibs_painter](https://github.com/trivial-space/trivalibs/tree/main/crates/trivalibs_painter) rendering lib to ease the use of WGPU.

## shader-crates

Shaders are compiled from Rust to rspirv shaders using [cargo-gpu](https://github.com/Rust-GPU/cargo-gpu).

To compile a shader crate, run `cargo gpu build` in the crate directory.

To watch compile, install `watchexec` and run `watchexec -r -e rs cargo gpu build` in the shader crate directory.

These crates can also be used from CPU Rust code, just like any other crate.
