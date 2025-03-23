#![allow(unused_imports)]

use core::f32::consts::TAU;

use spirv_std::glam::{vec3, Vec2, Vec4};
use spirv_std::num_traits::Float;
use trivalibs_shaders::color::{hsv2rgb, rgb2hsl};
// use crate::fbm;

pub fn color_test(uv: Vec2, time: f32) -> Vec4 {
	let c = vec3(uv.x, uv.y, 1.0);
	let mut c_hsv = rgb2hsl(c);
	c_hsv.z = (c_hsv.z + (time + uv.x * TAU * 6.0).sin()) * 0.5;
	let c_rgb = hsv2rgb(c_hsv);
	c_rgb.extend(1.0)
	// c.extend(1.0)
}
