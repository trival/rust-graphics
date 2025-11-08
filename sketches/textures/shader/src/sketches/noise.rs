use crate::{book_of_shaders::shapes::rounded_rect_smooth, utils::aspect_preserving_uv};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use spirv_std::{
	Image, Sampler,
	glam::{UVec2, Vec2, Vec3, Vec4, mat2, vec2, vec3},
};
use trivalibs_nostd::{
	bits::FloatBits,
	float_ext::FloatExt,
	random::{
		hash::hash2d,
		simplex::{rot_noise_2d, simplex_noise_2d, simplex_noise_3d},
	},
	vec_ext::VecExt,
};

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
fn fbm(st: Vec2, num_octaves: usize) -> f32 {
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

pub fn fbm_shader(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let st = aspect_preserving_uv(uv, size) * 3.0;

	let time = time;

	let q = vec2(
		// simplex_noise_3d(st.extend(time)).fit1101(),
		// simplex_noise_3d((st + vec2(1.0, 0.0)).extend(time)).fit1101(),
		fbm(st, 5),
		fbm(st + vec2(10.0, 10.0), 5),
	);

	let r = vec2(
		// simplex_noise_3d((st + 1.0 * q + vec2(1.7, 9.2)).extend(0.15 * time)).fit1101(),
		// simplex_noise_3d((st + 1.0 * q + vec2(8.3, 2.8)).extend(0.126 * time)).fit1101(),
		fbm(st + 1.1 * q + 0.15 * time, 5),
		fbm(st + 1.0 * q + 0.126 * time, 5),
	);

	// let f = simplex_noise_2d(st + r).fit1101();
	let f = fbm(st + r + time * 0.1, 5);

	let mut color = vec3(0.101961, 0.619608, 0.666667).lerp(
		vec3(0.666667, 0.666667, 0.198039),
		((f * f) * 4.0).clamp01(),
	);

	color = color.lerp(vec3(0.0, 0.0, 0.164706), q.length().clamp01());

	color = color.lerp(vec3(0.666667, 1.0, 1.0), r.length().clamp01());

	((f * f * f + 0.6 * f * f + 0.5 * f) * color).extend(1.0)
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

	let curr_x_pos = ((curr_x * LINE_COUNT).frct() - 0.5) / 3.;
	let prev_x_pos = ((prev_x * LINE_COUNT).frct() - 0.5) / 3. + 1. / 3.;
	let next_x_pos = ((next_x * LINE_COUNT).frct() - 0.5) / 3. - 1. / 3.;

	let get_line = |x: f32| x.smoothstep(-0.06, -0.04) * x.smoothstep(0.06, 0.04);

	let offset = 0.45 * noise;

	let line_curr = get_line(curr_x_pos + offset);
	let line_prev = get_line(prev_x_pos + offset);
	let line_next = get_line(next_x_pos + offset);

	let bg = Vec3::splat((noise + 1.0) / 4.0);

	let color = if (uv_current.x * LINE_COUNT).frct() < 0.02 {
		Vec3::ZERO
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

pub fn noisy_squares(uv: Vec2, _size: Vec2, _time: f32) -> Vec4 {
	let idx = (uv * 3.0).floor() + 1.0;
	let tile_uv = (uv * 3.0).frct().fit0111();

	let quad_size = hash2d(idx.to_bits()) * 0.6 + 0.9;

	let fbm = |scale: f32, fade: f32, num_octaves: usize| {
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

	let noise = fbm(idx.x + 1.1, 1.2 / idx.y, 5);

	let square = rounded_rect_smooth(
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
