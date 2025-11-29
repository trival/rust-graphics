#![allow(unexpected_cfgs)]

#[cfg(not(target_arch = "spirv"))]
use glam::Vec2;
#[cfg(target_arch = "spirv")]
use spirv_std::glam::Vec2;
use trivalibs_nostd::prelude::*;

pub fn rounded_rect(st: Vec2, center: Vec2, size: Vec2, radius: f32) -> f32 {
	let offset = size / 2. - radius;
	let d = ((st - center).abs() - offset).max(Vec2::ZERO).length();
	radius.step(d)
}

pub fn rounded_rect_smooth(
	st: Vec2,
	center: Vec2,
	size: Vec2,
	radius: f32,
	smoothness: f32,
) -> f32 {
	let offset = size / 2. - radius;
	let d = ((st - center).abs() - offset).max(Vec2::ZERO).length();

	let s = smoothness / 2.;
	let e0 = radius + s;
	let e1 = radius - s;

	d.smoothstep(e0, e1)
}
