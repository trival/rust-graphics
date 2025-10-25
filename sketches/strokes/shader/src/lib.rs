#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec3, Vec4, vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use trivalibs_shaders::{float_ext::FloatExt, vec_ext::VecExt};

// Line vertex shader
#[spirv(vertex)]
pub fn line_vert(
	// Attribs
	position: Vec2,
	width: f32,
	_length: f32,
	uv: Vec2,
	local_uv: Vec2,
	// Uniforms
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &Vec2,
	// Outputs
	#[spirv(position)] out_pos: &mut Vec4,
	out_uv: &mut Vec2,
	out_local_uv: &mut Vec2,
	out_width: &mut f32,
) {
	let pos = (position / size).fit0111();

	*out_pos = vec4(pos.x, -pos.y, 0.0, 1.0);
	*out_uv = uv;
	*out_local_uv = local_uv;
	*out_width = width;
}

// Line fragment shader with brush effect (simplified without noise texture for now)
#[spirv(fragment)]
pub fn line_frag(
	uv: Vec2,
	local_uv: Vec2,
	_width: f32,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] color: &Vec3,
	out: &mut Vec4,
) {
	// Simulate brush stroke with edge fading
	// Using similar logic to old shader but without noise texture

	let mut alpha = 0.8; // Base alpha

	// Fade out edges along the stroke width (localUv.y)
	let edge_fade_y = 1.0 - local_uv.y.fit0111().abs().powf(10.0);
	alpha *= edge_fade_y;

	// Fade out edges along stroke length (uv.y)
	let edge_fade_x = 1.0 - uv.y.fit0111().abs().powf(10.0);
	alpha *= edge_fade_x;

	// Fade out at the end of stroke
	let end_fade_val = 1.85 - uv.x;
	let end_fade = if end_fade_val < 1.0 {
		end_fade_val.powf(3.0)
	} else {
		1.0
	};
	alpha *= end_fade;

	// Clamp alpha
	let clamped_alpha = (alpha * 0.6).clamp01();

	*out = color.extend(clamped_alpha);
}

// Background shader - simple solid color
#[spirv(fragment)]
pub fn bg_frag(
	_uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] color: &Vec3,
	out: &mut Vec4,
) {
	*out = color.extend(1.0);
}
