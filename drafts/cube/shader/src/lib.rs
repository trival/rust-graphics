#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::{
	glam::{Vec2, Vec3, Vec4},
	spirv, Image, Sampler,
};

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	color: Vec3,
	uv: Vec2,
	#[spirv(position)] clip_pos: &mut Vec4,
	out_color: &mut Vec3,
	out_uv: &mut Vec2,
) {
	*out_color = color;
	*out_uv = uv;
	*clip_pos = position.extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(
	in_color: Vec3,
	in_uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	frag_color: &mut Vec4,
) {
	let col = tex.sample(*sampler, in_uv);
	*frag_color = col * in_color.extend(1.0);
}
