#![allow(unused_imports)]

use crate::utils::{flip_y, smoothstep};
use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec4};
use spirv_std::num_traits::Float;
use trivalibs_shaders::step::{step, Step};

pub fn rect(size: Vec2, center: Vec2, st: Vec2) -> f32 {
	let half_size = size * 0.5;
	let rect = (center - half_size).step(st) * (1.0 - (center + half_size).step(st));
	rect.x * rect.y
}

pub fn smooth_rect(size: Vec2, center: Vec2, st: Vec2, radius: f32) -> f32 {
	let half_size = size * 0.5;
	let half_radius = radius * 0.5;

	let left = center.x - half_size.x;
	let le1 = left + half_radius;
	let le2 = left - half_radius;

	let right = center.x + half_size.x;
	let re1 = right + half_radius;
	let re2 = right - half_radius;

	let x = smoothstep(le2, le1, st.x) - smoothstep(re2, re1, st.x);

	let bottom = center.y - half_size.y;
	let be1 = bottom + half_radius;
	let be2 = bottom - half_radius;

	let top = center.y + half_size.y;
	let te1 = top + half_radius;
	let te2 = top - half_radius;

	let y = smoothstep(be2, be1, st.y) - smoothstep(te2, te1, st.y);

	x * y
}

pub fn rect_shader(st: Vec2) -> Vec4 {
	let st = flip_y(st);

	// let bl = step(0.25, st.x) * step(0.25, st.y);
	// let tr = step(0.25, 1.0 - st.x) * step(0.25, 1.0 - st.y);

	// let val = bl * tr;

	// let vert = step(0.25, st.x) * step(0.25, 1.0 - st.x);
	// let horz = step(0.25, st.y) * step(0.25, 1.0 - st.y);

	let size = vec2(0.5, 0.25);
	let center1 = vec2(0.5, 0.3);
	let rec1 = rect(size, center1, st);

	let center2 = vec2(0.5, 0.7);
	let rec2 = smooth_rect(size, center2, st, 0.15);

	let color1 = vec3(rec1, rec1, 0.0);
	let color2 = vec3(rec2, 0.0, rec2);

	(color1 + color2).extend(1.0)
}
