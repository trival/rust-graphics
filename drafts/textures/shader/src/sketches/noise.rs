#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{
	glam::{mat2, uvec2, vec2, vec3, UVec2, Vec2, Vec3, Vec4},
	Image, Sampler,
};
use trivalibs_shaders::{
	bits::FloatBits,
	fit::Fit,
	random::{
		hash::{hash, hash21, hash2d, hash3d, hashi},
		simplex::{simplex_noise_2d, simplex_noise_3d},
	},
	smoothstep::Smoothstep,
};

use crate::utils::aspect_preserving_uv;

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
	let st = aspect_preserving_uv(uv, size) * 3.0;

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

pub fn hash_test(uv: Vec2, time: f32) -> Vec4 {
	let q_uv = (uv * 2.).fract();
	let q_idx = (uv * 2.).floor().as_uvec2();

	let color = if uv.x > 0.98 || uv.y > 0.98 || uv.x < 0.02 || uv.y < 0.02 {
		Vec3::ZERO
	} else if q_uv.x > 0.98 || q_uv.y > 0.98 || q_uv.x < 0.02 || q_uv.y < 0.02 {
		Vec3::ZERO
	} else if q_idx.eq(&uvec2(0, 0)) {
		let v = hash(q_uv.x.to_bits() + hashi((q_uv.y + time).to_bits()));
		vec3(v, 0.0, 0.0)
	} else if q_idx.eq(&uvec2(1, 0)) {
		let v = hash21((q_uv + time).to_bits());
		vec3(0.0, v, 0.0)
	} else if q_idx.eq(&uvec2(0, 1)) {
		let v = hash2d((q_uv + time).to_bits());
		v.extend(1.0)
	} else if q_idx.eq(&uvec2(1, 1)) {
		hash3d(q_uv.extend(time).to_bits())
	} else {
		vec3(0.0, 1.0, 1.0)
	};

	color.extend(1.0)
}

const LINE_COUNT: f32 = 30.;

pub fn noisy_lines_1(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv_current = aspect_preserving_uv(uv, size);

	let noise =
		simplex_noise_3d((uv_current * vec2(2.5, 1.5) - vec2(0., time * 0.6)).extend(time * 0.2));

	let x_offset = 1.0 / LINE_COUNT;
	let curr_x = uv_current.x;
	let next_x = curr_x + x_offset;
	let prev_x = curr_x - x_offset;
	// fract() does not work with negative numbers.
	// Without this the first column would have no previous (green) line.
	let prev_x = if prev_x < 0.0 { 1.0 + prev_x } else { prev_x };

	let curr_x_pos = ((curr_x * LINE_COUNT).fract() - 0.5) / 3.;
	let prev_x_pos = ((prev_x * LINE_COUNT).fract() - 0.5) / 3. + 1. / 3.;
	let next_x_pos = ((next_x * LINE_COUNT).fract() - 0.5) / 3. - 1. / 3.;

	let get_line = |x: f32| x.smoothstep(-0.06, -0.04) * x.smoothstep(0.06, 0.04);

	let offset = 0.45 * noise;

	let line_curr = get_line(curr_x_pos + offset);
	let line_prev = get_line(prev_x_pos + offset);
	let line_next = get_line(next_x_pos + offset);

	let bg = Vec3::splat((noise + 1.0) / 4.0);

	let color = if (uv_current.x * LINE_COUNT).fract() < 0.02 {
		Vec3::new(0.0, 0.0, 0.0)
	} else if line_curr > 0.0 {
		bg.lerp(Vec3::new(1.0, 1.0, 1.0), line_curr)
	} else if line_next > 0.0 {
		bg.lerp(Vec3::new(0.0, 1.0, 1.0), line_next)
	} else if line_prev > 0.0 {
		bg.lerp(Vec3::new(0.0, 1.0, 0.0), line_prev)
	} else {
		bg
	};
	// bg.lerp(Vec3::ONE, line_curr + line_prev + line_next);

	color.extend(1.0)
}
