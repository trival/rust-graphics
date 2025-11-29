#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{UVec2, Vec2, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{Image, Sampler, spirv};

mod book_of_shaders;
mod shaders;
mod utils;

#[spirv(fragment)]
pub fn simplex_prefilled(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(descriptor_set = 1, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	out: &mut Vec4,
) {
	*out = shaders::simplex_prefilled::shader(uv, tex, sampler, size);
}

#[spirv(fragment)]
pub fn fbm_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = shaders::fbm::shader(uv, *size, *time);
}

#[spirv(fragment)]
pub fn noisy_lines_1(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = shaders::noisy_lines_1::shader(uv, *size, *time);
}

#[spirv(fragment)]
pub fn noisy_lines_2(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = shaders::noisy_line::shader(uv, *size, *time);
}

#[spirv(fragment)]
pub fn circular_line(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = shaders::circular_line::shader(uv, *size, *time);
}

#[spirv(fragment)]
pub fn tiled_lines(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = shaders::tiled_lines::shader(uv, *size, *time);
}

#[spirv(fragment)]
pub fn net(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] _time: &f32,
	out: &mut Vec4,
) {
	*out = shaders::net::shader(uv, *size);
}

#[spirv(fragment)]
pub fn noisy_quads(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = shaders::noisy_squares::shader(uv, size.as_vec2(), *time);
}

#[spirv(fragment)]
pub fn bos_shapes_rounded_rect(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] _time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::rounded_rect::shader(uv);
}

#[spirv(fragment)]
pub fn bos_shaping_fns(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] _time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::shaping_fns::shader(uv);
}

#[spirv(fragment)]
pub fn bos_shapes_rect(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] _time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::rect::shader(uv);
}

#[spirv(fragment)]
pub fn bos_shapes_circle(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::circle::shader(uv, *time);
}

#[spirv(fragment)]
pub fn bos_colors(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::colors::shader(uv, *time);
}

#[spirv(fragment)]
pub fn bos_shapes_circles(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::circles::shader(uv, *time);
}
