#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{vec2, vec4, UVec2, Vec2, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 1, binding = 0)] time: &f32,
	out: &mut Vec4,
) {
	let tile_size = vec2(6.0, 12.0);
	let gap_size = tile_size * 0.02;
	let mut tile = uv * tile_size;
	let y_offet = tile.y.floor() % 2.0;
	if y_offet == 1.0 {
		tile.x += 0.5;
	}
	tile.y -= (time * 0.5).fract();
	tile -= gap_size * 0.5;
	let tile = tile - tile.floor();
	*out = if tile.x >= 1.0 - gap_size.x || tile.y >= 1.0 - gap_size.y {
		vec4(0.4, 0.6, 0.9, 1.0)
	} else {
		vec4(1.0, 0.8, 0.5, 1.0)
	}
}
