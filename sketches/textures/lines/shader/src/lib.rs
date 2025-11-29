#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{UVec2, Vec2, Vec4};
use spirv_std::spirv;

pub mod sketches;

#[spirv(fragment)]
pub fn noisy_lines_2(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = sketches::lines::noisy_lines_2(uv, *size, *time);
}
