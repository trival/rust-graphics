use spirv_std::glam::{vec3, Vec2, Vec4};
use spirv_std::num_traits::Float;
use trivalibs_shaders::color::{hsl2rgb, rgb2hsl};
// use crate::fbm;

pub fn color_test(uv: Vec2, time: f32) -> Vec4 {
	let c = vec3(uv.x, uv.y, 1.0);
	let mut c_hsl = rgb2hsl(c);
	// c_hsl.z = (c_hsl.z + (time + uv.x * 2.0).sin()) * 0.5;
	let c_rgb = hsl2rgb(c_hsl);
	c_rgb.extend(1.0);
	c.extend(1.0)
}
