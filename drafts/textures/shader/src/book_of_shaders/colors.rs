#![allow(unused_imports)]

use core::f32::consts::TAU;

use spirv_std::glam::{vec2, vec3, Vec2, Vec4};
use spirv_std::num_traits::Float;
use trivalibs_shaders::color::{
	hsv2rgb, hsv2rgb_smooth, hsv2rgb_smoother, hsv2rgb_smoothest, rgb2hsl,
};
use trivalibs_shaders::coords::PolarCoord;

use super::shapes::circle_smooth;
// use crate::fbm;

pub fn color_test(uv: Vec2, time: f32) -> Vec4 {
	let c = vec3(uv.x, uv.y, 1.0);
	let mut c_hsv = rgb2hsl(c);
	c_hsv.z = (c_hsv.z + (time + uv.x * TAU * 6.0).sin()) * 0.5;
	let c_rgb = hsv2rgb(c_hsv);

	let center = vec2(0.5, 0.5);
	let radius = 0.4;

	let circle = circle_smooth(center, radius, uv, 0.05);

	let polar = PolarCoord::from_2d_with_center(uv, center);

	let c_polar = hsv2rgb_smooth(vec3(polar.angle / TAU + 0.5, polar.radius / radius, 1.0));

	c_rgb.lerp(c_polar, 1.0 - circle).extend(1.0)
	// c.extend(1.0)
}
