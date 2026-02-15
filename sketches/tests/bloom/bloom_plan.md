# Bloom Effect Implementation Plan

## Overview

Implement a minimal proof-of-concept modern bloom effect in `sketches/tests/bloom/` using the painter library's mipmap capabilities. The bloom will use progressive downsampling and upsampling with Gaussian blur at each mip level, following the approach used by Bevy engine and described in the LearnOpenGL/CatlikeCoding tutorials.

## Core Architecture

### Single Layer with Mipmap Chain

- Use one bloom layer with 5 mip levels (0-4)
- Leverage `.with_mip_target()` and `.with_mip_source()` to target specific mip levels
- Each mip level is half the resolution of the previous (e.g., 1920×1080 → 960×540 → 480×270 → 240×135 → 120×67)

### Effect Pipeline

```
Scene Render (bright objects on dark background)
    ↓
Brightness Threshold → Extract bright pixels to bloom mip 0
    ↓
Downsample Chain → Blur + downsample: mip 0→1→2→3→4
    ↓
Upsample Chain → Blur + upsample with additive blending: mip 4→3→2→1→0
    ↓
Final Composite → Combine scene + bloom result
```

## File Structure

```
sketches/tests/bloom/
├── Cargo.toml
├── src/main.rs              # Main application with effect chain
└── shader/
    ├── Cargo.toml
    └── src/lib.rs           # 5 shader functions
```

## Shaders Required

1. **`test_scene_fs`** - Generate 5-6 circles with varying brightness (distance field):
   - 2-3 bright circles (HDR intensity 2.0-5.0) that will bloom
   - 2-3 dimmer circles (intensity 0.5-0.8) below threshold, no bloom
   - Dark background (0.1)
2. **`threshold_fs`** - Extract pixels with brightness > threshold (uniform binding)
3. **`downsample_blur_fs`** - Downsample with separable Gaussian blur (using `gaussian_blur_9`)
4. **`upsample_blur_fs`** - Upsample with separable Gaussian blur (same kernel)
5. **`composite_fs`** - Combine scene + bloom with intensity multiplier (uniform binding)

## Effect Chain Details

### Separable Blur Strategy

Each blur pass uses TWO effects (horizontal + vertical) for efficient separable Gaussian blur:

- Horizontal pass: `dir = vec2(1.0, 0.0)`
- Vertical pass: `dir = vec2(0.0, 1.0)`

### Downsample Chain (8 effects total)

For each transition mip N → N+1:

```rust
// Horizontal blur
.with_mip_source(N)
.with_mip_target(N+1)

// Vertical blur
.with_mip_source(N+1)
.with_mip_target(N+1)
```

Sequence: 0→1, 1→2, 2→3, 3→4

### Upsample Chain (8 effects total)

For each transition mip N → N-1:

```rust
// Horizontal blur with additive blending
.with_mip_source(N)
.with_mip_target(N-1)
.with_blend_state(ADDITIVE_BLEND)

// Vertical blur with additive blending
.with_mip_source(N-1)
.with_mip_target(N-1)
.with_blend_state(ADDITIVE_BLEND)
```

Sequence: 4→3, 3→2, 2→1, 1→0

**Additive Blend State:**

```rust
wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent::REPLACE,
}
```

## Key Implementation Details

### Layer Creation

```rust
// Scene layer (for test pattern) - Use single_effect_layer utility
let scene_layer = p.single_effect_layer(test_scene_shade)
    .with_bindings(vec![(0, u_time.binding())])
    .create();

// Bloom processing layer - This is where mipmaps are used
let bloom_layer = p.layer()
    .with_effects([
        threshold_effect,           // 1 effect (reads scene layer, writes bloom mip 0)
        /* downsample effects */,   // 8 effects (4 levels × 2 passes, all within bloom layer)
        /* upsample effects */,     // 8 effects (4 levels × 2 passes, all within bloom layer)
    ])
    .with_mips_max(5)
    .create();

// Final composite - Use single_effect_layer utility
let canvas = p.single_effect_layer(composite_shade)
    .with_bindings(vec![
        (0, u_bloom_intensity.binding()),
        (1, sampler_linear),
    ])
    .with_layers(vec![
        (0, scene_layer.binding()),
        (1, bloom_layer.binding()),
    ])
    .create();
```

### Resolution Management

Each mip level needs its own resolution binding for blur calculations:

```rust
fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
    self.resolution_mip0.update(p, vec2(width as f32, height as f32));
    self.resolution_mip1.update(p, vec2((width/2) as f32, (height/2) as f32));
    self.resolution_mip2.update(p, vec2((width/4) as f32, (height/4) as f32));
    self.resolution_mip3.update(p, vec2((width/8) as f32, (height/8) as f32));
}
```

### Sampler Configuration

```rust
let sampler = p.sampler()
    .with_filters(wgpu::FilterMode::Linear)
    .with_mipmap_filter(wgpu::FilterMode::Linear)
    .create();
```

## Implementation Steps

### Phase 1: Project Setup (30 min)

1. Create directory structure at `sketches/tests/bloom/`
2. Create `Cargo.toml` with workspace dependencies (trivalibs, bytemuck)
3. Create shader `Cargo.toml` with spirv-std and trivalibs_nostd
4. Create stub `main.rs` with CanvasApp structure
5. Create stub `shader/src/lib.rs`

### Phase 2: Test Scene (30 min)

1. Implement `test_scene_fs` with 5-6 circles using distance fields:
   - 2-3 HDR bright circles (intensity 2.0-5.0) for bloom demonstration
   - 2-3 dimmer circles (intensity 0.5-0.8) to show threshold working
   - Dark background (0.1)
2. Create scene layer (no mipmaps) and render directly to verify
3. Add time binding for optional animation

### Phase 3: Threshold Extraction (30 min)

1. Implement `threshold_fs` shader (brightness filter)
2. Create bloom layer with threshold effect targeting mip 0
3. Test by rendering bloom layer directly

### Phase 4: Downsample Chain (90 min)

1. Implement `downsample_blur_fs` using `gaussian_blur_9` from `trivalibs_nostd::blur`
2. Create resolution bindings for each mip level
3. Create 8 downsample effects (4 transitions × 2 passes each)
4. Test by sampling different mip levels to verify progressive downsampling

### Phase 5: Upsample Chain (90 min)

1. Implement `upsample_blur_fs` (same as downsample)
2. Create 8 upsample effects with additive blending
3. Test by rendering bloom mip 0 to see accumulated result

### Phase 6: Final Composite (30 min)

1. Implement `composite_fs` shader
2. Create canvas layer with composite effect
3. Bind scene and bloom layers to composite
4. Verify final output shows scene with bloom glow

### Phase 7: Polish & Tuning (60 min)

1. Verify threshold and bloom intensity can be adjusted at runtime via uniform bindings
2. Test with different parameter values to find good defaults
3. Add optional debug visualization (keyboard shortcuts to show different stages)
4. Document recommended parameter ranges in code comments

## Critical Files to Reference

1. **`trivalibs/examples/render_to_mip/main.rs`** - Mip-level targeting pattern
2. **`trivalibs/crates/trivalibs_nostd/src/blur.rs`** - Gaussian blur functions
3. **`trivalibs/examples/blur/main.rs`** - Multi-pass blur setup
4. **`trivalibs/examples/blur/shader/src/lib.rs`** - Blur shader implementation example
5. **`trivalibs/examples/dynamic_texture/main.rs`** - Shows `single_effect_layer` utility usage

## Key Technical Considerations

1. **Scene Layer Has No Mipmaps**: The scene layer renders at full resolution only. We sample it once in the threshold effect, then all blur operations happen within the bloom layer's mipmap chain.

2. **Bloom Layer Mipmap Chain**: All downsample and upsample operations read and write within the bloom layer's mipmaps (0-4). Never sample scene layer mipmaps.

3. **No Automatic Target Swapping**: Effects with `.with_mip_target()` don't trigger automatic render target swapping - this is desired behavior for building mipmap chains

4. **Effect Ordering is Critical**: Threshold (reads scene) → all downsamples (bloom mips only) → all upsamples (bloom mips only) must be in correct sequence

5. **Additive Blending Only for Upsampling**: Downsample uses REPLACE blend (default), upsample uses additive to accumulate bloom contributions

6. **Separable Blur Efficiency**: Using horizontal + vertical passes reduces texture samples significantly compared to 2D kernel

7. **Runtime-Adjustable Parameters**: Implement bloom threshold and intensity as uniform bindings (BindingBuffer<f32>) for easy runtime tweaking

8. **Use `single_effect_layer` Utility**: For layers that only render a single fragment effect (scene and canvas layers), use `p.single_effect_layer()` for simpler setup

## Success Criteria

- Bright objects have visible, smooth bloom glow without artifacts
- No visible seams or mip transition issues
- Runs at 60+ FPS at 1920×1080
- Clean, well-structured code following existing painter patterns
- Easy to adjust parameters (threshold, intensity) for tuning

## Estimated Timeline

Total: ~5.5 hours for complete POC (simplified test scene saves time)

## Out of Scope (Future Enhancements)

- Soft threshold with knee parameter
- Energy-conserving blend mode
- HDR tone mapping
- Lens dirt mask overlay
- Performance optimizations (compute shaders, smaller kernels)
