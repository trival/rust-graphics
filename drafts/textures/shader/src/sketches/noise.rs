#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{
	glam::{mat2, vec2, vec3, UVec2, Vec2, Vec4},
	Image, Sampler,
};
use trivalibs_shaders::{
	fit::Fit,
	random::simplex::{simplex_noise_2d, simplex_noise_3d},
};

use crate::utils::st_from_uv_size;

pub fn simplex_shader(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let aspect = size.x as f32 / size.y as f32;
	let uv = uv * vec2(aspect, 1.0);
	let uv = uv * 10.0;
	let noise = simplex_noise_3d(uv.extend(time)).fit1101();
	// let noise = simplex_noise_2d(uv * time.sin()).fit1101();
	Vec4::new(noise, noise, noise, 1.0)
}

pub fn simplex_prefilled(
	uv: Vec2,
	tex: &Image!(2D, type=f32, sampled),
	sampler: &Sampler,
	size: &UVec2,
) -> Vec4 {
	let aspect = size.x as f32 / size.y as f32;
	let noise = tex.sample(*sampler, vec2(uv.x * aspect * 3., uv.y * 3.));
	let val = (noise.x + noise.y * 0.5 + noise.z * 0.25 + noise.w * 0.125) / 1.875;
	// let val = noise.w;
	Vec4::new(val, val, val, 1.0)
}

// Fbm shader ported from https://thebookofshaders.com/13/

const NUM_OCTAVES: usize = 5;

fn fbm(st: Vec2) -> f32 {
	let mut v = 0.0;
	let mut a = 0.5;
	let mut st = st;
	let shift = vec2(100.0, 100.0);
	// Rotate to reduce axial bias
	let rot = mat2(vec2(0.5.cos(), 0.5.sin()), vec2(-0.5.sin(), 0.5.cos()));
	for _ in 0..NUM_OCTAVES {
		v += a * simplex_noise_2d(st).fit1101();
		st = rot * st * 2.0 + shift;
		a *= 0.5;
	}
	v
}

pub fn fbm_shader(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let st = st_from_uv_size(uv, size) * 3.0;

	let time = time;

	let q = vec2(
		// simplex_noise_3d(st.extend(time)).fit1101(),
		// simplex_noise_3d((st + vec2(1.0, 0.0)).extend(time)).fit1101(),
		fbm(st),
		fbm(st + vec2(10.0, 10.0)),
	);

	let r = vec2(
		// simplex_noise_3d((st + 1.0 * q + vec2(1.7, 9.2)).extend(0.15 * time)).fit1101(),
		// simplex_noise_3d((st + 1.0 * q + vec2(8.3, 2.8)).extend(0.126 * time)).fit1101(),
		fbm(st + 1.1 * q + 0.15 * time),
		fbm(st + 1.0 * q + 0.126 * time),
	);

	// let f = simplex_noise_2d(st + r).fit1101();
	let f = fbm(st + r + time * 0.1);

	let mut color = vec3(0.101961, 0.619608, 0.666667).lerp(
		vec3(0.666667, 0.666667, 0.198039),
		((f * f) * 4.0).clamp(0.0, 1.0),
	);

	color = color.lerp(vec3(0.0, 0.0, 0.164706), q.length().clamp(0.0, 1.0));

	color = color.lerp(vec3(0.666667, 1.0, 1.0), r.length().clamp(0.0, 1.0));

	((f * f * f + 0.6 * f * f + 0.5 * f) * color).extend(1.0)
}
