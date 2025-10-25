#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Mat3A, Mat4, Vec2, Vec3, Vec4, vec3};
use spirv_std::spirv;
use trivalibs_shaders::vec_ext::VecExt;

#[spirv(vertex)]
pub fn ground_vert(
	position: Vec3,
	uv: Vec2,
	normal: Vec3,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] m_mat: &Mat4,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] n_mat: &Mat3A,
	#[spirv(uniform, descriptor_set = 0, binding = 2)] vp_mat: &Mat4,
	#[spirv(position)] out_pos: &mut Vec4,
	out_norm: &mut Vec3,
	out_uv: &mut Vec2,
) {
	*out_pos = *vp_mat * *m_mat * position.extend(1.0);
	*out_norm = *n_mat * normal;
	*out_uv = uv;
}

#[spirv(fragment)]
pub fn ground_frag(in_norm: Vec3, in_uv: Vec2, out: &mut Vec4) {
	let uv = in_uv * 40.0;
	let uv = uv.fract();

	let col = if uv.x < 0.2 || uv.y < 0.2 {
		in_norm.fit1101()
	} else {
		vec3(in_uv.x, in_uv.y, 0.5)
	};

	*out = col.powf(2.2).extend(1.0);
}
