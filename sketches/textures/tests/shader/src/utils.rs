use spirv_std::glam::{Vec2, mat2, vec2};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_nostd::{prelude::*, random::simplex::simplex_noise_2d};

// Geometry utilities used by multiple shaders

pub fn rect(size: Vec2, center: Vec2, st: Vec2) -> f32 {
	let half_size = size * 0.5;
	let rect = (center - st).abs() / half_size;
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
	let dist = (st - center).length_squared() / radius;
	radius.step(dist)
}

pub fn circle_smooth(center: Vec2, radius: f32, st: Vec2, smoothness: f32) -> f32 {
	let dist = (st - center).length();
	dist.smoothstep(radius - smoothness, radius + smoothness)
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

// Noise utilities

pub fn fbm(st: Vec2, num_octaves: usize) -> f32 {
	let mut v = 0.0;
	let mut a = 0.5;
	let mut st = st;
	let shift = vec2(100.0, 100.0);
	// Rotate to reduce axial bias
	let rot = mat2(vec2(0.5.cos(), 0.5.sin()), vec2(-0.5.sin(), 0.5.cos()));
	for _ in 0..num_octaves {
		v += a * simplex_noise_2d(st).fit1101();
		st = rot * st * 2.0 + shift;
		a *= 0.5;
	}
	v
}
