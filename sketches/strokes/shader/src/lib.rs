#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec3, Vec4, vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use trivalibs_shaders::{float_ext::FloatExt, random::simplex::fbm_simplex_2d, vec_ext::VecExt};

// Line vertex shader
#[spirv(vertex)]
pub fn line_vert(
	// Attribs
	position: Vec2,
	_width: f32,
	_length: f32,
	uv: Vec2,
	local_uv: Vec2,
	// Uniforms
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &Vec2,
	// Outputs
	#[spirv(position)] out_pos: &mut Vec4,
	out_uv: &mut Vec2,
	out_local_uv: &mut Vec2,
) {
	let pos = (position / size).fit0111();

	*out_pos = vec4(pos.x, -pos.y, 0.0, 1.0);
	*out_uv = uv;
	*out_local_uv = local_uv;
}

// Line fragment shader with brush effect (simplified without noise texture for now)
#[spirv(fragment)]
pub fn line_frag(
	uv: Vec2,
	local_uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] color: &Vec3,
	out: &mut Vec4,
) {
	let mut alpha = fbm_simplex_2d(uv, 4, 2.0, 1.0) * 0.5;

	// Fade out edges along the stroke width (localUv.y)
	alpha -= local_uv.x.fit0111().abs().powf(10.0);

	// Fade out edges along stroke length (uv.y)
	alpha -= uv.y.fit0111().abs().powf(10.0);

	// Fade out at the end of stroke
	alpha *= 1.0.min((1.85 - uv.x).powf(3.0));

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
