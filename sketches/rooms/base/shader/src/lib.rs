#![no_std]
#![allow(unexpected_cfgs)]

use shared_nostd::flip_y;
use spirv_std::glam::{Mat4, Vec2, Vec3, Vec4, swizzles::*, vec2, vec3};
use spirv_std::{Image, Sampler, spirv};
use trivalibs_nostd::prelude::*;
use trivalibs_nostd::random::simplex::{rot_noise_3d, simplex_noise_3d};

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
	*out_uv = uv;
	*out_norm = normal;
}

#[spirv(fragment)]
pub fn wall_pre_render_frag(in_pos: Vec3, in_uv: Vec2, _in_norm: Vec3, out: &mut Vec4) {
	// let noise1 = simplex_noise_3d(in_pos / 1. + Vec3::splat(140.0));
	let noise1 = rot_noise_3d(in_pos / 3. + Vec3::splat(140.0), 0.).0;

	// let noise2 = rot_noise_3d(in_pos.cross(in_norm), in_uv.y);

	// let val = (noise1.0 + noise2.0).fit1101() / 2.0;
	let val = noise1.fit1101() / 1.0;

	*out = Vec3::splat(val.clamp01().powf(2.2)).extend(1.0);
	// *out = vec3(val.clamp01(), in_uv.x, in_uv.y).extend(1.0);
	// *out = vec3(0.0, in_uv.x, in_uv.y).extend(1.0);
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
	*out_uv = uv;
	*out_norm = normal;
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

	// let col = if uv.x < 0.75 || uv.y < 0.75 {
	// 	tex.sample(*sampler, vec2(in_uv.x, 1.0 - in_uv.y)).xyz()
	// } else {
	// 	vec3(0.0, in_uv.x, in_uv.y)
	// };
	let col = tex.sample(*sampler, flip_y(in_uv)).xyz();

	*out = col.xyz().powf(2.2).extend(1.0);
}
