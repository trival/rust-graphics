# Experiments with WGPU and Rust-GPU

This repository contains some experiments with WGPU and Rust-GPU. It utilizes the work-in-progress [trivalibs_painter](https://github.com/trivial-space/trivalibs/tree/main/crates/trivalibs_painter) rendering lib to ease the use of WGPU.

## shader-crates

Shaders are compiled from Rust to rspirv shaders using [cargo-gpu](https://github.com/Rust-GPU/cargo-gpu).

To compile a shader crate, run `cargo gpu build` in the crate directory.

To watch compile, install `watchexec` and run `watchexec -r -e rs cargo gpu build` in the shader crate directory.

These crates can also be used from CPU Rust code, just like any other crate.

## !IMPORTANT!

For the time beeing, because of a libm bug, use following command to build the shader crate:

```bash
cargo gpu build --spirv-builder-source "https://github.com/Rust-GPU/rust-gpu" --spirv-builder-version "1e4e468ccf7965d90a9748c7513f72e852fb5041" --multimodule
# and
watchexec -r -e rs cargo gpu build --spirv-builder-source "https://github.com/Rust-GPU/rust-gpu" --spirv-builder-version "1e4e468ccf7965d90a9748c7513f72e852fb5041" --multimodule
```
