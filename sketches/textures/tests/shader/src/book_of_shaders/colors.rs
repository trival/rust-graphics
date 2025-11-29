#![allow(unused_imports)]

use crate::utils;
use core::f32::consts::TAU;
use spirv_std::glam::{Vec2, Vec4, vec2, vec3};
use spirv_std::num_traits::Float;
use trivalibs_nostd::color::{
	hsv2rgb, hsv2rgb_smooth, hsv2rgb_smoother, hsv2rgb_smoothest, rgb2hsl,
};
use trivalibs_nostd::coords::PolarCoord;

pub fn shader(uv: Vec2, time: f32) -> Vec4 {
	let c = vec3(uv.x, uv.y, 1.0);
	let mut c_hsv = rgb2hsl(c);
	c_hsv.z = (c_hsv.z + (time + uv.x * TAU * 6.0).sin()) * 0.5;
	let c_rgb = hsv2rgb(c_hsv);

	let center = vec2(0.5, 0.5);
	let radius = 0.4;

	let circle = utils::circle_smooth(center, radius, uv, 0.05);

	let polar = PolarCoord::from_2d_with_center(uv, center);
	let c_polar = hsv2rgb_smooth(vec3(
		polar.angle / TAU + 0.5,
		polar.radius / (radius - 0.01),
		1.0,
	));

	c_polar.lerp(c_rgb, circle).extend(1.0)
	// c.extend(1.0)
}
