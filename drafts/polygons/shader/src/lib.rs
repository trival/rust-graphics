#![no_std]
#![allow(unexpected_cfgs)]

use glam::{Vec3, Vec4};
use spirv_std::spirv;
#[cfg(not(target_arch = "spirv"))]
use trival_painter::macros::*;

#[cfg(not(target_arch = "spirv"))]
#[apply(gpu_data)]
pub struct Vertex {
	pub position: Vec3,
	pub color: Vec3,
}

#[spirv(vertex)]
pub fn vertex(
	position: Vec3,
	color: Vec3,
	#[spirv(position)] clip_pos: &mut Vec4,
	out_color: &mut Vec3,
) {
	*out_color = color;
	*clip_pos = position.extend(1.0);
}

#[spirv(fragment)]
pub fn fragment(in_color: Vec3, frag_color: &mut Vec4) {
	*frag_color = in_color.extend(1.0);
}
