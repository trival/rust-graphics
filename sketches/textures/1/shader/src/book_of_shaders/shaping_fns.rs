#![allow(unused_imports)]

use crate::utils::flip_y;
use core::f32::consts::PI;
use spirv_std::num_traits::Float;
use spirv_std::{
	glam::{Vec2, Vec3, Vec4, vec3},
	num_traits::Pow,
};
use trivalibs_nostd::prelude::*;

// fn plot_line_1(st: Vec2) -> f32 {
// 	smoothstep(0.02, 0.0, (st.y - st.x).abs())
// }

fn plot(st: Vec2, val: f32) -> f32 {
	// smoothstep(val - 0.02, val, st.y) - smoothstep(val, val + 0.02, st.y)
	step(val - 0.02, st.y) - step(val + 0.02, st.y)
}

const PLOT_COLOR: Vec3 = vec3(0.0, 1.0, 0.0);

pub fn shaping_fns(st: Vec2) -> Vec4 {
	let st = flip_y(st);

	let x = st.x;

	// let y = x;
	// let y = x.pow(5.0);
	// let y = step(0.4, x) - step(0.6, x);
	// let y = x.pow(5.0) - st.y.pow(5.0);
	// let y = x.log(0.5);
	// let y = x.sqrt();
	// let y = x.pow(0.4);
	// let y = st.x * 0.5 + 0.5;
	// let y = x.pow(PI);
	// let y = (x * 16.0).sin().abs();
	// let y = 1.0 - (x * 16.0).sin().abs();
	// let y = (x * 20.0).sin() * 0.5 + 1.0;
	let y = (x * 8.0).sin().frct();
	// let y = (x * 8.0).sin();

	let color = Vec3::splat(y);

	let pct = plot(st, y);
	let color = (1.0 - pct) * color + pct * PLOT_COLOR;

	// gamma correction
	let color = color.powf(2.2);

	color.extend(1.0)
}
