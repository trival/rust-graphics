#![no_std]

use spirv_std::{
	glam::{Vec3, Vec4},
	spirv,
};

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	color: Vec3,
	normal: Vec3,
	#[spirv(position)] clip_pos: &mut Vec4,
	out_color: &mut Vec3,
	out_norm: &mut Vec3,
) {
	*out_color = color;
	*out_norm = normal;
	*clip_pos = position.extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(in_color: Vec3, _in_norm: Vec3, frag_color: &mut Vec4) {
	*frag_color = in_color.extend(1.0);
}
