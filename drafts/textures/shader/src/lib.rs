#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec2, UVec2, Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};
use trivalibs_shaders::noise::simplex::simplex_noise_2d;

#[spirv(fragment)]
pub fn simplex_shader_frag(uv: Vec2, frag_color: &mut Vec4) {
	let noise = simplex_noise_2d(uv * 10.);
	*frag_color = Vec4::new(noise, noise, noise, 1.0);
}

#[spirv(fragment)]
pub fn simplex_prefilled_frag(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 0, binding = 2)] size: &UVec2,
	frag_color: &mut Vec4,
) {
	let aspect = size.x as f32 / size.y as f32;
	let noise = tex.sample(*sampler, vec2(uv.x * aspect * 3., uv.y * 3.));
	let val = (noise.x + noise.y + noise.z + noise.w) / 4.0;
	// let val = noise.w;
	*frag_color = Vec4::new(val, val, val, 1.0);
}
