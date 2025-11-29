use crate::utils;
use core::f32::consts::TAU;
use shared_nostd::flip_y;
use spirv_std::glam::{Vec2, Vec3, Vec4, vec2, vec3};
use trivalibs_nostd::coords::PolarCoord;
use trivalibs_nostd::prelude::*;

pub fn shader(st: Vec2, time: f32) -> Vec4 {
	let st = flip_y(st);

	let center = vec2(0.5, 0.5);
	let radius = (time * 0.8).sin() * 0.06 + 0.3;

	let circle1 = utils::circle(center, radius, st);
	let circle2 = utils::circle_smooth(center - vec2(0.0, 0.01), radius, st, 0.05);

	let color1 = vec3(1.0, 1.0, 0.0);
	let color2 = vec3(1.0, 0.0, 0.0);
	let color_shadow = vec3(0.3, 0.3, 0.1);

	let bg_color = Vec3::splat(1.0);

	let color = if circle1 > 0.0 {
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
