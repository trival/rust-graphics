use core::f32::consts::{PI, TAU};
use spirv_std::glam::{Vec2, Vec3, Vec4, vec2};
use trivalibs_nostd::{prelude::*, random::hash::hash};

fn circle_line(i: f32, line_count: f32, t: f32, angle: f32) -> f32 {
	let v1 = hash(i as u32 * 3) * 0.5;
	let v2 = hash(i as u32 + line_count as u32) * 0.5 + 0.5;

	let s = if v2 > 0.75 { 1.0 } else { -1.0 };
	let a = (angle + t * v1 * (0.8 / (i.powf(0.5))) * s).frct().abs();
	step(v1, a) * step(a, v2)
}

pub fn shader(st: Vec2, t: f32) -> Vec4 {
	let c1 = (Vec2::splat(0.4) - st).length();
	let c2 = (Vec2::splat(0.6) - st).length();

	let line_count = 35.0_f32.powf(2.0);
	let val = (c1 * c2).powf(0.92);

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

	color.extend(1.0)
}
