//! Ported to Rust from <https://github.com/Tw1ddle/Sky-Shader/blob/master/src/shaders/glsl/sky.fragment>
#![allow(unexpected_cfgs)]
#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

use glam::{vec2, vec4, Vec2, Vec4};
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main_fs(output: &mut Vec4) {
	*output = vec4(0.0, 0.0, 1.0, 1.0);
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
	// Create a "full screen triangle" by mapping the vertex index.
	// ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
	let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
	let pos = 2.0 * uv - Vec2::ONE;

	*builtin_pos = pos.extend(0.0).extend(1.0);
}
