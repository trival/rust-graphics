use crate::utils::aspect_preserving_uv;
use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{float_ext::FloatExt, random::simplex::simplex_noise_2d};

pub fn noisy_lines_2(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv_current = aspect_preserving_uv(uv, size);

	let uv = uv_current * 2.0 - 1.0;

	let noise = simplex_noise_2d(vec2(uv.y * 0.3, time * 0.5)).fit1101();

	let y = uv.y * 1.01;
	let caps = 1.0 - y.abs().smoothstep(1.01, 0.60);
	let caps = 1.0.lerp(50.0, caps.powf(20.0));

	let x = uv.x * 5.0 * caps + noise * 1.5 * caps;

	let line = x.abs().smoothstep(1.01, 0.99);

	let col_bg = Vec3::ONE;
	let col_line = vec3(0.2, 0.2, 0.2);

	let col = col_bg.lerp(col_line, line);

	col.powf(2.2).extend(1.0)
}
