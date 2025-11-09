#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{UVec3, Vec2, Vec3, Vec4, vec4};
use spirv_std::spirv;
use trivalibs_nostd::prelude::*;

#[spirv(fragment)]
pub fn main(
	coord: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec3,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] color: &Vec3,
	out: &mut Vec4,
) {
	*out = vec4(
		color.x,
		(color.y - coord.x).clamp01(),
		(color.z - coord.y).clamp01(),
		1.0,
	);
}
