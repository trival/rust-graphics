# Experiments with WGPU and Rust-GPU

## shader-crates

Some crates are compiled to rspirv shaders using [cargo-gpu](https://github.com/Rust-GPU/cargo-gpu).

To compile, run `cargo gpu build` in the crate directory.

Unfortunately this does not work with the workspace `glam` dependency. Thus
`glam` needs to be defined separately in the `Cargo.toml` for the spirv target.

These crates can also be used from CPU Rust code, just like any other crate.
