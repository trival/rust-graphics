#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Mat4, Vec2, Vec3, Vec4, swizzles::*, vec3};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{Image, Sampler, spirv};
use trivalibs_nostd::prelude::*;
use trivalibs_nostd::random::simplex::rot_noise_3d;

#[spirv(vertex)]
pub fn wall_pre_render_vert(
	position: Vec3,
	uv: Vec2,
	normal: Vec3,
	#[spirv(position)] out_vert: &mut Vec4,
	out_pos: &mut Vec3,
	out_uv: &mut Vec2,
	out_norm: &mut Vec3,
) {
	*out_vert = uv.fit0111().extend(0.).extend(1.0);
	*out_pos = position;
	*out_norm = normal;
	*out_uv = uv;
}

#[spirv(fragment)]
pub fn wall_pre_render_frag(in_pos: Vec3, in_uv: Vec2, in_norm: Vec3, out: &mut Vec4) {
	let noise1 = rot_noise_3d(in_pos, in_uv.x);

	let noise2 = rot_noise_3d(in_pos.cross(in_norm), in_uv.y);

	let val = (noise1.0 + noise2.0).fit1101() / 2.0;
	// let val = noise1.0.fit1101() / 2.0;

	*out = Vec3::splat(val.clamp01().powf(2.2)).extend(1.0);
}

#[spirv(vertex)]
pub fn wall_render_vert(
	position: Vec3,
	uv: Vec2,
	normal: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] mvp_mat: &Mat4,
	#[spirv(position)] out_pos: &mut Vec4,
	out_uv: &mut Vec2,
	out_norm: &mut Vec3,
) {
	*out_pos = *mvp_mat * position.extend(1.0);
	*out_norm = normal;
	*out_uv = uv;
}

#[spirv(fragment)]
pub fn wall_render_frag(
	in_uv: Vec2,
	_in_norm: Vec3,
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	let uv = in_uv * 40.0;
	let uv = uv.fract();

	let col = if uv.x < 0.25 || uv.y < 0.25 {
		tex.sample(*sampler, in_uv).xyz()
	} else {
		vec3(in_uv.x, in_uv.y, 0.5)
	};

	*out = col.xyz().powf(2.2).extend(1.0);
}
