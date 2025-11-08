#![allow(unused_imports)]

use core::f32::consts::{PI, TAU};

use crate::utils::flip_y;
use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4, vec2, vec3};
use spirv_std::num_traits::Float;
use trivalibs_nostd::coords::PolarCoord;
use trivalibs_nostd::prelude::*;
use trivalibs_nostd::random::hash::hash;

pub fn rect(size: Vec2, center: Vec2, st: Vec2) -> f32 {
	let half_size = size * 0.5;

	// This is the first strategy of the book of shaders
	// let rect = (center - half_size).step(st) * (1.0 - (center + half_size).step(st));
	// rect.x * rect.y

	let rect = (center - st).abs() / half_size;
	// 1.0 - (rect.x.max(rect.y)).clamp(0.0, 1.0).floor()
	1.0.step(rect.x.max(rect.y))
}

pub fn rect_smooth(size: Vec2, center: Vec2, st: Vec2, smoothness: f32) -> f32 {
	let half_size = size * 0.5;

	let rect = (center - st).abs() / half_size;

	let s = smoothness / size;
	let e0 = Vec2::ONE + s;
	let e1 = Vec2::ONE - s;

	let smooth = rect.smoothstep(e0, e1);

	smooth.x * smooth.y
}

pub fn circle(center: Vec2, radius: f32, st: Vec2) -> f32 {
	// let dist = (st - center).length();
	let dist = (st - center).length_squared() / radius; // eqivalent (why?) and omits the sqrt of length()
	radius.step(dist) // inverting edge and x results in 1.0 - x.step(egde)
}

pub fn circle_smooth(center: Vec2, radius: f32, st: Vec2, smoothness: f32) -> f32 {
	let dist = (st - center).length();
	dist.smoothstep(radius - smoothness, radius + smoothness)
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
	bg_color
		.lerp(color2, rec2) //
		.lerp(color1, rec1)
		.extend(1.0)
}

pub fn circle_shader(st: Vec2, time: f32) -> Vec4 {
	let st = flip_y(st);

	let center = vec2(0.5, 0.5);
	let radius = (time * 0.8).sin() * 0.06 + 0.3;

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
		polar.angle = ((polar.angle + time * 0.2) / (TAU / 2.0)).frct() * TAU / 1.0;
		let uv = polar.to_2d();

		let cell_uv = (uv * 6.0).frct();
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

pub fn shader_circles(st: Vec2, t: f32) -> Vec4 {
	let c1 = (Vec2::splat(0.4) - st).length();
	let c2 = (Vec2::splat(0.6) - st).length();

	let line_count = 35.0.powf(2.0);
	// let val = c1 + c2;
	// let val = (c1 + c2) / 2.;
	// let val = c1 - c2;
	let val = (c1 * c2).powf(0.92);
	// let val = (c1 * c2);
	// let val = c1.min(c2);
	// let val = c1.max(c2);
	// let val = c1.powf(c2);
	// let val = c2.powf(c1);

	let i = (val * line_count).powf(0.5).floor();

	let c1 = Vec3::ZERO;
	let c2 = Vec3::ONE;

	let center = vec2(0.5, 0.5) - st;
	let angle = (center.y.atan2(center.x) + PI) / TAU;

	let color = if i < 6.0 {
		let line = circle_line(6.0, line_count, t, angle);
		c2.lerp(c1, line)
	} else {
		let line = circle_line(i, line_count, t, angle);
		c1.lerp(c2, line)
	};

	return color.extend(1.0);
}

fn circle_line(i: f32, line_count: f32, t: f32, angle: f32) -> f32 {
	let v1 = hash(i as u32 * 3) * 0.5;
	let v2 = hash(i as u32 + line_count as u32) * 0.5 + 0.5;

	let s = if v2 > 0.75 { 1.0 } else { -1.0 };
	let a = (angle + t * v1 * (0.8 / (i.powf(0.5))) * s).frct().abs();
	step(v1, a) * step(a, v2)
}

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

pub fn rounded_rect_shader(st: Vec2) -> Vec4 {
	let uv = (st * 3.0).frct().fit0111();
	let idx_v2 = (st * 3.0).floor();
	let idx = (idx_v2.x + idx_v2.y * 3.0) / 9.0;

	let size = vec2(1.2, 1.0);

	let center = vec2((idx * TAU).cos(), (idx * TAU).sin()) * 0.3;

	let rec = rounded_rect_smooth(uv, center, size, idx * 0.7, 0.3);

	let color = vec3(0.1, 0.0, 0.0);

	let bg_color = Vec3::splat(1.0);
	bg_color.lerp(color, rec).extend(1.0)
}
