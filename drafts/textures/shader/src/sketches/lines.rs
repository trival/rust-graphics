use crate::utils::aspect_preserving_uv;
use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{float_ext::FloatExt, random::simplex::simplex_noise_2d, vec_ext::VecExt};

pub fn noisy_lines_2(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv_current = aspect_preserving_uv(uv, size);

	let uv = uv_current * 2.0 - 1.0;

	let bend_noise = simplex_noise_2d(vec2(uv.y * 0.6, time * 0.5)).fit1101();

	let texture_noise =
		(simplex_noise_2d((uv_current + vec2(bend_noise / (15.0 / 2.0), 0.0)) * vec2(92.0, 02.0))
			+ simplex_noise_2d(
				(uv_current + vec2(bend_noise / (15.0 / 2.0), 0.0)) * vec2(92.0, 2.0) * 2.0 + 120.,
			) * 0.5)
			/ 1.5;

	let grid = (uv_current * 340.).sin().fit1101() * 0.5 + 0.5;
	let bg_texture_noise =
		((simplex_noise_2d(uv_current * 200.) + simplex_noise_2d(uv_current * 400. + 12.) * 0.5) / 1.5)
			* grid.x
			* grid.y;

	let y = uv.y * 0.96;
	let caps = y.abs().step_fn(1.0, 0.75, |t| t.powf(0.25)) + 0.01;

	let x = uv.x * 15.0 / caps + bend_noise * 3.5 / caps;

	let line = x.abs().smoothstep(1.0, 0.8)
		* y.abs().smoothstep(1.0, 0.95)
		* ((texture_noise
			.fit1101()
			.powf(0.2 + y.abs().step_fn(0.88, 1.0, |t| t.powf(3.)) * 9.)
			+ bg_texture_noise * 0.1)
			/ 1.1)
			.clamp01()
			.powf(1.5);

	let col_bg = Vec3::ONE;
	let col_line = vec3(0.2, 0.2, 0.2);

	let col = col_bg.lerp(col_line, line);

	col.powf(2.2).extend(1.0)
}
