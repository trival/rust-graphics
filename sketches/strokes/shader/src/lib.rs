#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec4, vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use trivalibs_shaders::float_ext::{FloatExt, fit0111};

// Line vertex shader
#[spirv(vertex)]
pub fn line_vert(
	position: Vec2,
	width: f32,
	_length: f32,
	uv: Vec2,
	local_uv: Vec2,
	#[spirv(position)] out_pos: &mut Vec4,
	out_uv: &mut Vec2,
	out_local_uv: &mut Vec2,
	out_width: &mut f32,
) {
	// Transform from pixel coordinates (0-1200) to normalized device coordinates (-1 to 1)
	// fit0111: x => x * 2.0 - 1.0
	let canvas_size = 1200.0;
	let normalized_pos = position / canvas_size;
	let clip_pos_x = fit0111(normalized_pos.x);
	let clip_pos_y = -fit0111(normalized_pos.y); // Flip Y axis

	*out_pos = vec4(clip_pos_x, clip_pos_y, 0.0, 1.0);
	*out_uv = uv;
	*out_local_uv = local_uv;
	*out_width = width;
}

// Manual pow implementation for f32
fn pow_f32(base: f32, exp: f32) -> f32 {
	// For integer exponents, use repeated multiplication
	if exp == 3.0 {
		return base * base * base;
	} else if exp == 10.0 {
		let b2 = base * base;
		let b4 = b2 * b2;
		let b8 = b4 * b4;
		return b8 * b2;
	}
	// Fallback: just return base for now
	base
}

// Line fragment shader with brush effect (simplified without noise texture for now)
#[spirv(fragment)]
pub fn line_frag(uv: Vec2, local_uv: Vec2, _width: f32, out: &mut Vec4) {
	// Simulate brush stroke with edge fading
	// Using similar logic to old shader but without noise texture

	let mut alpha = 0.8; // Base alpha

	// Fade out edges along the stroke width (localUv.y)
	let edge_fade_y = 1.0 - pow_f32(local_uv.y.fit0111().abs(), 10.0);
	alpha *= edge_fade_y;

	// Fade out edges along stroke length (uv.y)
	let edge_fade_x = 1.0 - pow_f32(uv.y.fit0111().abs(), 10.0);
	alpha *= edge_fade_x;

	// Fade out at the end of stroke
	let end_fade_val = 1.85 - uv.x;
	let end_fade = if end_fade_val < 1.0 {
		pow_f32(end_fade_val, 3.0)
	} else {
		1.0
	};
	alpha *= end_fade;

	// Clamp alpha
	let clamped_alpha = (alpha * 0.6).clamp01();

	// Test with varying color based on position to verify the pipeline works
	// TODO: Pass actual tile colors per-tile
	// Simple HSV-like color variation using UV coordinates
	let h = (local_uv.x + uv.y) % 1.0;
	let h6 = h * 6.0;

	let (r, g, b) = if h6 < 1.0 {
		(1.0, h6, 0.0)
	} else if h6 < 2.0 {
		(2.0 - h6, 1.0, 0.0)
	} else if h6 < 3.0 {
		(0.0, 1.0, h6 - 2.0)
	} else if h6 < 4.0 {
		(0.0, 4.0 - h6, 1.0)
	} else if h6 < 5.0 {
		(h6 - 4.0, 0.0, 1.0)
	} else {
		(1.0, 0.0, 6.0 - h6)
	};

	*out = vec4(r, g, b, clamped_alpha);
}

// Background shader - simple solid color
#[spirv(fragment)]
pub fn bg_frag(_uv: Vec2, out: &mut Vec4) {
	// Light gray background
	*out = vec4(0.9, 0.9, 0.9, 1.0);
}
