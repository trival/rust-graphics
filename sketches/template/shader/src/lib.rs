#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{Vec2, Vec4, vec4};
use spirv_std::spirv;

#[spirv(fragment)]
pub fn main(coord: Vec2, out: &mut Vec4) {
	*out = vec4(1.0, coord.x, coord.y, 1.0);
}
