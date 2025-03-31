#![no_std]
#![allow(unexpected_cfgs)]

use spirv_std::glam::{UVec2, Vec2, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{spirv, Image, Sampler};

mod book_of_shaders;
pub mod sketches;
pub mod utils;

#[spirv(fragment)]
pub fn simplex_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = sketches::noise::simplex_shader(uv, *size, *time);
}

#[spirv(fragment)]
pub fn simplex_prefilled(
	uv: Vec2,
	#[spirv(descriptor_set = 0, binding = 0)] tex: &Image!(2D, type=f32, sampled),
	#[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
	#[spirv(uniform, descriptor_set = 0, binding = 2)] size: &UVec2,
	out: &mut Vec4,
) {
	*out = sketches::noise::simplex_prefilled(uv, tex, sampler, size);
}

#[spirv(fragment)]
pub fn fbm_shader(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = sketches::noise::fbm_shader(uv, *size, *time);
}

#[spirv(fragment)]
pub fn bos_shaping_fns(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] _time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::shaping_fns::shaping_fns(uv);
}

#[spirv(fragment)]
pub fn bos_shapes_rect(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] _time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::shapes::rect_shader(uv);
}

#[spirv(fragment)]
pub fn bos_shapes_circle(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::shapes::circle_shader(uv, *time);
}

#[spirv(fragment)]
pub fn bos_colors(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::colors::color_test(uv, *time);
}

#[spirv(fragment)]
pub fn bos_shapes_circles(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = book_of_shaders::shapes::shader_circles(uv, *time);
}

#[spirv(fragment)]
pub fn hash_test(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] _size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = sketches::noise::hash_test(uv, *time);
}

#[spirv(fragment)]
pub fn tiled_plates(
	uv: Vec2,
	#[spirv(uniform, descriptor_set = 0, binding = 0)] size: &UVec2,
	#[spirv(uniform, descriptor_set = 0, binding = 1)] time: &f32,
	out: &mut Vec4,
) {
	*out = sketches::tiles::tiled_plates(uv, *size, *time);
}
