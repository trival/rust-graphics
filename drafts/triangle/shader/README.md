# shader-crate

install [cargo-gpu](https://github.com/Rust-GPU/cargo-gpu) and run
`cargo gpu build` to build the shader.

Unfortunately the it does not work with the workspace `glam` dependency. Thus
`glam` needs to be defined separately in the `Cargo.toml`.

This crate can also be used from CPU Rust code, just like any other crate.
