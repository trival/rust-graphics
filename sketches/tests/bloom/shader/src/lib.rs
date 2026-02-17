#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec3, Vec4, Vec4Swizzles, vec2, vec3, vec4};
use spirv_std::spirv;
use spirv_std::{Image, Sampler};
#[allow(unused_imports)]
use trivalibs_nostd::prelude::*;

/// Test scene with multiple circles at varying brightness levels
#[spirv(fragment)]
pub fn test_scene_fs(
	#[spirv(frag_coord)] frag_coord: Vec4,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] resolution: &Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	// Use normalized coordinates
	let uv = frag_coord.xy() / *resolution;
	let aspect = resolution.x / resolution.y;

	// Adjust UV for aspect ratio
	let uv_corrected = vec2(uv.x * aspect, uv.y);

	// Dark background
	let mut color = vec3(0.1, 0.1, 0.1);

	// Helper function to create a circle with smooth edges
	let circle = |center: Vec2, radius: f32, brightness: Vec3| {
		let dist = (uv_corrected - center).length();
		let alpha = 1.0 - ((dist - radius) / (radius * 0.05)).clamp(0.0, 1.0);
		brightness * alpha
	};

	let t = *time * 0.5;

	// Bright circles (HDR intensity - will bloom)
	color += circle(vec2(0.3 + t.sin() * 0.1, 0.5), 0.15, vec3(3.0, 3.0, 2.5)); // Yellow-white
	color += circle(vec2(0.7, 0.5 + t.cos() * 0.1), 0.12, vec3(1.5, 2.5, 4.0)); // Cyan
	color += circle(vec2(0.5, 0.3), 0.1, vec3(4.0, 1.5, 1.5)); // Red-orange

	// Dimmer circles (below threshold - won't bloom)
	color += circle(vec2(0.25, 0.75), 0.08, vec3(0.6, 0.6, 0.8)); // Dim blue
	color += circle(vec2(0.75, 0.75), 0.08, vec3(0.8, 0.6, 0.6)); // Dim red
	color += circle(vec2(0.5, 0.7), 0.06, vec3(0.7, 0.7, 0.5)); // Dim yellow

	*out = vec4(color.x, color.y, color.z, 1.0).powf(2.3);
}

/// Extract bright pixels above threshold
#[spirv(fragment)]
pub fn threshold_fs(
	#[spirv(frag_coord)] frag_coord: Vec4,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] resolution: &Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] threshold: &f32,
	#[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let uv = frag_coord.xy() / *resolution;
	let color = tex.sample(*sampler, uv);

	// Calculate luminance
	let brightness = color.x * 0.2126 + color.y * 0.7152 + color.z * 0.0722;

	if brightness > *threshold {
		*out = color;
	} else {
		*out = vec4(0.0, 0.0, 0.0, 1.0);
	}
}

/// Downsample with Gaussian blur (single pass for simplicity)
#[spirv(fragment)]
pub fn downsample_blur_fs(
	#[spirv(frag_coord)] frag_coord: Vec4,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] resolution: &Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] blur_radius: &f32,
	#[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let uv = frag_coord.xy() / *resolution;
	// Simple box blur for downsampling (4 samples)
	let offset = vec2(0.5, 0.5) * *blur_radius / *resolution;
	let s0 = tex.sample(*sampler, uv + vec2(-offset.x, -offset.y));
	let s1 = tex.sample(*sampler, uv + vec2(offset.x, -offset.y));
	let s2 = tex.sample(*sampler, uv + vec2(-offset.x, offset.y));
	let s3 = tex.sample(*sampler, uv + vec2(offset.x, offset.y));
	*out = (s0 + s1 + s2 + s3) * 0.25;
}

/// Upsample with blur (single pass)
#[spirv(fragment)]
pub fn upsample_blur_fs(
	#[spirv(frag_coord)] frag_coord: Vec4,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] resolution: &Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] blur_radius: &f32,
	#[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let uv = frag_coord.xy() / *resolution;
	// Tent filter (9 samples) for upsampling
	let offset = vec2(1.0, 1.0) * *blur_radius / *resolution;

	let s = tex.sample(*sampler, uv);
	let n = tex.sample(*sampler, uv + vec2(0.0, offset.y));
	let s_s = tex.sample(*sampler, uv + vec2(0.0, -offset.y));
	let e = tex.sample(*sampler, uv + vec2(offset.x, 0.0));
	let w = tex.sample(*sampler, uv + vec2(-offset.x, 0.0));

	let ne = tex.sample(*sampler, uv + vec2(offset.x, offset.y));
	let nw = tex.sample(*sampler, uv + vec2(-offset.x, offset.y));
	let se = tex.sample(*sampler, uv + vec2(offset.x, -offset.y));
	let sw = tex.sample(*sampler, uv + vec2(-offset.x, -offset.y));

	*out = s * 0.25 + (n + s_s + e + w) * 0.125 + (ne + nw + se + sw) * 0.0625;
}

/// Composite scene with bloom
#[spirv(fragment)]
pub fn composite_fs(
	#[spirv(frag_coord)] frag_coord: Vec4,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] resolution: &Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] bloom_intensity: &f32,
	#[spirv(descriptor_set = 0, binding = 2)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] scene_tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 1, binding = 1)] bloom_tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let uv = frag_coord.xy() / *resolution;
	let scene = scene_tex.sample(*sampler, uv);
	let bloom = bloom_tex.sample(*sampler, uv);

	*out = scene + bloom * *bloom_intensity;
}
