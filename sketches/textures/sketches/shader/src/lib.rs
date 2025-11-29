#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{UVec2, Vec2, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

pub mod sketches;

#[spirv(fragment)]
pub fn tiled_plates(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = sketches::tiles::tiled_plates(uv, *size, *time);
}

#[spirv(fragment)]
pub fn pool_tiles(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = sketches::misc::pool_tiles(uv, *size, *time);
}
