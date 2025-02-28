#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec4, UVec2, Vec2, Vec4};
use spirv_std::{spirv, Image, Sampler};
use trivalibs_shaders::noise::simplex::simplex_noise_2d;

#[spirv(fragment)]
pub fn main(
	uv: Vec2,
	// #[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	// #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	frag_color: &mut Vec4,
) {
	let noise = simplex_noise_2d(uv * 10.);
	*frag_color = Vec4::new(noise, noise, noise, 1.0);
}
