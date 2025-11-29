use crate::utils;
use spirv_std::glam::{Vec2, Vec4, vec2, vec3};
use trivalibs_nostd::{
	prelude::*,
	random::{hash::hash2d, simplex::rot_noise_2d},
};

pub fn shader(uv: Vec2, _size: Vec2, _time: f32) -> Vec4 {
	let idx = (uv * 3.0).floor() + 1.0;
	let tile_uv = (uv * 3.0).frct().fit0111();

	let quad_size = hash2d(idx.to_bits()) * 0.6 + 0.9;

	let fbm_local = |scale: f32, fade: f32, num_octaves: usize| {
		let mut noise = 0.0;
		let mut a = 1.0;
		let mut st = uv;
		let shift = vec2(100.0, 100.0);
		let rot = (1.0 / (num_octaves as f32 * 2.0 + 1.0)) * 2.0;
		for i in 0..num_octaves {
			noise += a * rot_noise_2d(st, rot * i as f32).0.fit1101();
			st = st * scale + shift;
			a *= fade;
		}
		noise
	};

	let noise = fbm_local(idx.x + 1.1, 1.2 / idx.y, 5);

	let square = utils::rounded_rect_smooth(
		tile_uv * (noise * 0.12 + 0.88),
		Vec2::ZERO,
		quad_size,
		0.03,
		0.01,
	);

	let col1 = vec3(0.1, 0.2, 0.3);
	let col2 = vec3(0.45, 0.5, 0.55);

	let col = col1.lerp(col2, square);

	col.powf(2.2).extend(1.0)
}
