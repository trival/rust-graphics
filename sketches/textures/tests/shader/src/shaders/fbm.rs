use crate::utils;
use shared_nostd::aspect_preserving_uv;
use spirv_std::glam::{UVec2, Vec2, Vec4, vec2, vec3};
use trivalibs_nostd::prelude::*;

pub fn shader(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let st = aspect_preserving_uv(uv, size) * 3.0;

	let q = vec2(utils::fbm(st, 5), utils::fbm(st + vec2(10.0, 10.0), 5));

	let r = vec2(
		utils::fbm(st + 1.1 * q + 0.15 * time, 5),
		utils::fbm(st + 1.0 * q + 0.126 * time, 5),
	);

	let f = utils::fbm(st + r + time * 0.1, 5);

	let mut color = vec3(0.101961, 0.619608, 0.666667).lerp(
		vec3(0.666667, 0.666667, 0.198039),
		((f * f) * 4.0).clamp01(),
	);

	color = color.lerp(vec3(0.0, 0.0, 0.164706), q.length().clamp01());

	color = color.lerp(vec3(0.666667, 1.0, 1.0), r.length().clamp01());

	((f * f * f + 0.6 * f * f + 0.5 * f) * color).extend(1.0)
}
