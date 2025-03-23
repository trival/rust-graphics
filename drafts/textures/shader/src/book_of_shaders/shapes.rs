#![allow(unused_imports)]

use core::f32::consts::TAU;

use crate::utils::flip_y;
use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec3, Vec4};
use spirv_std::num_traits::Float;
use trivalibs_shaders::coords::PolarCoord;
use trivalibs_shaders::smoothstep::smoothstep;
use trivalibs_shaders::step::{step, Step};

pub fn rect(size: Vec2, center: Vec2, st: Vec2) -> f32 {
	let half_size = size * 0.5;
	let rect = (center - half_size).step(st) * (1.0 - (center + half_size).step(st));
	rect.x * rect.y
}

pub fn rect_smooth(size: Vec2, center: Vec2, st: Vec2, radius: f32) -> f32 {
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

pub fn circle(center: Vec2, radius: f32, st: Vec2) -> f32 {
	let dist = (st - center).length();
	dist.step(radius)
}

pub fn circle_smooth(center: Vec2, radius: f32, st: Vec2, smoothness: f32) -> f32 {
	let dist = (st - center).length();
	smoothstep(radius - smoothness, radius + smoothness, dist)
}

pub fn rect_shader(st: Vec2) -> Vec4 {
	let st = flip_y(st);

	// let bl = step(0.25, st.x) * step(0.25, st.y);
	// let tr = step(0.25, 1.0 - st.x) * step(0.25, 1.0 - st.y);

	// let val = bl * tr;

	// let vert = step(0.25, st.x) * step(0.25, 1.0 - st.x);
	// let horz = step(0.25, st.y) * step(0.25, 1.0 - st.y);

	let size = vec2(0.8, 0.65);

	let center1 = vec2(0.5, 0.50);
	let rec1 = rect(size, center1, st);

	let center2 = vec2(0.5, 0.485);
	let rec2 = rect_smooth(size, center2, st, 0.12);

	let color1 = vec3(1.0, 1.0, 0.0);
	let color2 = vec3(0.3, 0.3, 0.1);

	let bg_color = Vec3::splat(1.0);
	bg_color.lerp(color2, rec2).lerp(color1, rec1).extend(1.0)
}

pub fn circle_shader(st: Vec2, time: f32) -> Vec4 {
	let st = flip_y(st);

	let center = vec2(0.5, 0.5);
	let radius = 0.3;

	let circle1 = circle(center, radius, st);
	let circle2 = circle_smooth(center - vec2(0.0, 0.01), radius, st, 0.05);

	let color1 = vec3(1.0, 1.0, 0.0);
	let color2 = vec3(1.0, 0.0, 0.0);
	let color_shadow = vec3(0.3, 0.3, 0.1);

	let bg_color = Vec3::splat(1.0);

	let color = if circle1 > 0.0 {
		// color1
		let uv = (st - center) / (radius * 2.0);
		let mut polar = PolarCoord::from_2d(uv);
		polar.angle = ((polar.angle + time * 0.2) / (TAU / 2.0)).fract() * TAU / 1.0;
		let uv = polar.to_2d();

		let cell_uv = (uv * 6.0).fract();
		let cell = (uv * 6.0).floor();
		if cell_uv.x < 0.2 || cell_uv.y < 0.2 {
			Vec3::ZERO
		} else if (cell.x + cell.y) % 2.0 == 0.0 {
			color2
		} else {
			color1
		}
	} else {
		color_shadow.lerp(bg_color, circle2)
	};
	color.extend(1.0)
}
