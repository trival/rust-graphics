# Claude Code Guide for rust-graphics

This document explains how to create, compile, and run sketches in this project.

## Project Structure

- `sketches/` - Individual sketch projects
- `trivalibs/` - Core rendering framework and utilities
- `sketches/template/` - Template for creating new sketches

Each sketch in `sketches/` has:

- `Cargo.toml` - Main sketch package configuration
- `shader/` - Rust-GPU shader crate
  - `Cargo.toml` - Shader crate configuration
  - `src/lib.rs` - Shader implementations
- `src/` - Rust application code
  - `main.rs` - Application entry point using `CanvasApp` trait

## Creating a New Sketch

1. **Copy the template**:

   ```bash
   cp -r sketches/template sketches/your-sketch-name
   ```

2. **Update `sketches/your-sketch-name/Cargo.toml`**:

   ```toml
   [package]
   name = "your-sketch-name"
   edition.workspace = true
   version = "0.1.0"
   ```

3. **Update `sketches/your-sketch-name/shader/Cargo.toml`**:

   ```toml
   [package]
   name = "your_sketch_name_shader"
   version = "0.1.0"
   edition.workspace = true
   ```

4. **Implement your sketch** in `src/main.rs` using the `CanvasApp` trait

## Compiling Shaders

Shaders are written in Rust using rust-gpu and compiled to SPIR-V.

**IMPORTANT**: Shader compilation must be run from the shader crate directory.

```bash
cd sketches/your-sketch-name/shader
cargo gpu build
```

This generates `.spv` files in the shader directory:

- `line_vert.spv`
- `line_frag.spv`
- `bg_frag.spv`
- etc.

These `.spv` files are loaded at runtime using macros like:

```rust
load_vertex_shader!(shade, p, "../shader/line_vert.spv");
load_fragment_shader!(shade, p, "../shader/line_frag.spv");
```

## Running a Sketch

From the project root directory:

```bash
cargo run --bin your-sketch-name
```

Or using the package name:

```bash
cargo run -p your-sketch-name
```

## Shader Development

### Available Shader Helpers

The `trivalibs_nostd` crate provides utilities for shader code:

#### float_ext

```rust
use trivalibs_nostd::float_ext::{fit0111, fit1101, FloatExt};
```

- `fit0111(x)` - Maps [0,1] to [-1,1]: `x * 2.0 - 1.0`
- `fit1101(x)` - Maps [-1,1] to [0,1]: `x * 0.5 + 0.5`
- `FloatExt` trait - Adds methods like `.clamp01()`, `.fit0111()`, `.fit1101()`, `.lerp()`, `.smoothen()`, `.smoothen_more()`, `.smoothstep()`, `.step()`, `.step_fn()`, `.frct()`, etc.

#### vec_ext

```rust
use trivalibs_nostd::vec_ext::VecExt;
```

Component-wise operations for Vec2, Vec3, Vec4:

- `.sin()`, `.cos()`, `.sqrt()`, `.frct()` - Trig and math functions
- `.fit0111()`, `.fit1101()`, `.clamp01()` - Range mapping
- `.smoothen()`, `.smoothen_more()`, `.smoothstep()` - Smooth interpolation
- `.step()`, `.step_f32()`, `.step_fn()` - Step functions
- `.lerp_vec()` - Component-wise lerp with vector t

#### color

```rust
use trivalibs_nostd::color::{rgb2hsl, hsv2rgb, hsv2rgb_smooth, hsv2rgb_smoother, hsv2rgb_smoothest};
```

- `rgb2hsl(c: Vec3) -> Vec3` - Convert RGB to HSL
- `hsv2rgb(c: Vec3) -> Vec3` - Convert HSV to RGB
- `hsv2rgb_smooth(c: Vec3) -> Vec3` - Smoothed HSV to RGB conversion
- `hsv2rgb_smoother(c: Vec3) -> Vec3` - Extra smooth HSV to RGB
- `hsv2rgb_smoothest(c: Vec3) -> Vec3` - Trigonometric smooth blend (expensive)

#### blur

```rust
use trivalibs_nostd::blur::{gaussian_blur, gaussian_blur_5, gaussian_blur_9, gaussian_blur_13, box_blur};
```

- `gaussian_blur(image, sampler, diameter, uv, res, dir)` - Separable Gaussian blur
- `gaussian_blur_5(image, sampler, uv, res, dir)` - Optimized 5-tap Gaussian (diameter 5.0)
- `gaussian_blur_9(image, sampler, uv, res, dir)` - Optimized 9-tap Gaussian (diameter 9.0)
- `gaussian_blur_13(image, sampler, uv, res, dir)` - Optimized 13-tap Gaussian (diameter 13.0)
- `box_blur(image, sampler, diameter, uv, res, dir)` - Separable box blur

For directional blur, use `dir: vec2(1.0, 0.0)` for horizontal, `vec2(0.0, 1.0)` for vertical.

#### coords

```rust
use trivalibs_nostd::coords::PolarCoord;
```

- `PolarCoord::from_2d(v: Vec2)` - Convert cartesian to polar coordinates
- `PolarCoord::from_2d_with_center(v: Vec2, center: Vec2)` - Convert with offset center
- `.to_2d()` - Convert polar back to cartesian
- `.as_vec()` - Get polar as Vec2 (radius, angle)

#### bits

```rust
use trivalibs_nostd::bits::FloatBits;
```

- `Vec2::to_bits() -> UVec2` / `Vec2::from_bits(UVec2)` - Float/bit conversion
- `Vec3::to_bits() -> UVec3` / `Vec3::from_bits(UVec3)` - For packing/unpacking
- `Vec4::to_bits() -> UVec4` / `Vec4::from_bits(UVec4)` - Useful for storage

### Shader Entry Points

Vertex shader:

```rust
#[spirv(vertex)]
pub fn my_vert(
    position: Vec2,
    #[spirv(position)] out_pos: &mut Vec4,
) {
    // Transform logic
}
```

Fragment shader:

```rust
#[spirv(fragment)]
pub fn my_frag(
    uv: Vec2,
    out: &mut Vec4,
) {
    // Color calculation
}
```

### Coordinate Transformations

Common pattern for converting pixel coordinates to NDC:

```rust
let canvas_size = 1200.0;
let normalized_pos = position / canvas_size;  // [0, 1]
let clip_pos_x = fit0111(normalized_pos.x);   // [-1, 1]
let clip_pos_y = -fit0111(normalized_pos.y);  // [-1, 1], flipped Y
*out_pos = vec4(clip_pos_x, clip_pos_y, 0.0, 1.0);
```

## Using trivalibs APIs

### Line Rendering

```rust
use trivalibs::rendering::line_2d::Line;

let line = Line::new(points, width, closed);
let geom: BufferedGeometry = line.to_buffered_geometry();

let form = p.form(&geom)
    .with_topology(wgpu::PrimitiveTopology::TriangleStrip)
    .create();
```

### Random Utilities

```rust
use trivalibs::utils::rand_utils::{
    rand_f32, rand_bool, rand_usize, rand_u32, rand_i32, Pick
};

let x = rand_f32();              // [0, 1)
let y = rand_usize(10);          // [0, 10)
let z = rand_bool(0.5);          // 50% true
let item = choices.pick_random(); // Random element from slice
```

### CanvasApp Trait

Main application structure:

```rust
impl CanvasApp<()> for App {
    fn init(p: &mut Painter) -> Self {
        // Initialize layers, shaders, shapes
        Self { /* fields */ }
    }

    fn update(&mut self, p: &mut Painter, tpf: f32) {
        // Update logic (called each frame)
        p.request_next_frame(); // For continuous rendering
    }

    fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
        // Render logic
        p.show(self.layer)
    }
}

fn main() {
    CanvasApp::<()>::run((), None);
}
```

## Common Workflows

### Development Cycle (Recommended)

Use the `dev` command for automatic hot-reload during development:

```bash
# From project root
cargo dev your-sketch
```

This automatically:
- Watches and rebuilds your Rust application code
- Watches and recompiles shaders with `cargo gpu build`
- Restarts the sketch after successful builds
- Shows all output with `[Main]` and `[Shader]` prefixes

The CanvasApp detects shader updates and hot-reloads them, so you see changes immediately.

### Manual Development Cycle

1. Edit shader code in `sketches/your-sketch/shader/src/lib.rs`
2. Compile shaders: `cd sketches/your-sketch/shader && cargo gpu build`
3. Edit application code in `sketches/your-sketch/src/`
4. Run from project root: `cargo run --bin your-sketch`

### Troubleshooting

**Shader compilation errors**:

- Make sure you're running `cargo gpu build` from the shader crate directory
- Check that spirv-std types are used (Vec2, Vec4, not glam types directly)
- spirv does support std library functions (e.g., `powf`, `cos`, `sin`) with the import of `spirv_std::num_traits::Float`

**Shader not found at runtime**:

- Verify `.spv` files exist in the shader directory
- Check the path in `load_vertex_shader!` / `load_fragment_shader!` macros
- Paths are relative to the source file that calls these macros, typically `"../shader/name.spv"`

**Geometry not rendering**:

- Verify shader vertex format matches geometry format
- Check topology setting (TriangleList, TriangleStrip, etc.)
- Ensure coordinate transformations map correctly to NDC [-1, 1]

## Performance Notes

- Use `p.request_next_frame()` in `update()` for continuous rendering
- Omit `request_next_frame()` for static scenes that only render once
- Typical frame rates: 1000-3000 FPS for simple scenes
