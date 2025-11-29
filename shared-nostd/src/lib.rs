#![no_std]
// Shared utilities for shader code (no_std compatible)
// This crate can be used by both shader crates and host code
#![allow(unexpected_cfgs)]

#[cfg(not(target_arch = "spirv"))]
use glam::{UVec2, Vec2, vec2};
#[cfg(target_arch = "spirv")]
use spirv_std::glam::{UVec2, Vec2, vec2};

pub mod shapes;

pub fn aspect_preserving_uv(uv: Vec2, size: UVec2) -> Vec2 {
	let aspect = size.x as f32 / size.y as f32;
	if aspect > 1.0 {
		uv * vec2(1.0, 1.0 / aspect)
	} else {
		uv * vec2(aspect, 1.0)
	}
}

pub fn flip_y(uv: Vec2) -> Vec2 {
	vec2(uv.x, 1.0 - uv.y)
}
