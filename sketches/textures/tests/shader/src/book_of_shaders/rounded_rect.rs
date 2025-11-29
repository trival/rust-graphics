use crate::utils;
use core::f32::consts::TAU;
use spirv_std::glam::{Vec2, Vec3, Vec4, vec2, vec3};
use trivalibs_nostd::prelude::*;

pub fn shader(st: Vec2) -> Vec4 {
	let uv = (st * 3.0).frct().fit0111();
	let idx_v2 = (st * 3.0).floor();
	let idx = (idx_v2.x + idx_v2.y * 3.0) / 9.0;

	let size = vec2(1.2, 1.0);

	let center = vec2((idx * TAU).cos(), (idx * TAU).sin()) * 0.3;

	let rec = utils::rounded_rect_smooth(uv, center, size, idx * 0.7, 0.3);

	let color = vec3(0.1, 0.0, 0.0);

	let bg_color = Vec3::splat(1.0);
	bg_color.lerp(color, rec).extend(1.0)
}
