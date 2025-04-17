use crate::utils::aspect_preserving_uv;
use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{float_ext::FloatExt, random::simplex::simplex_noise_3d};

const LINE_COUNT: f32 = 30.;

pub fn noisy_lines_2(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv_current = aspect_preserving_uv(uv, size);

	let noise1 =
		simplex_noise_3d((uv_current * vec2(4.5, -2.5) - vec2(0., time * 0.6)).extend(time * 0.2));
	let noise2 = simplex_noise_3d(
		(uv_current * vec2(4.5, -2.5) - vec2(0., time * 0.6)).extend(time * 0.2 + 0.3),
	);
	let noise3 =
		simplex_noise_3d((uv_current * vec2(6.5, 3.5) - vec2(0., time * 0.6)).extend(time * 0.2 + 0.6));

	let noise = (noise1 + noise2 + noise3 * 0.7) / 2.7;

	let x_offset = 1.0 / LINE_COUNT;
	let y_offset = 1.0 / LINE_COUNT;

	let mut color = Vec3::ZERO;

	for i in 0..LINE_COUNT as u32 {
		let y = (i as f32 + noise1).fract();
		let y_start = y * y_offset;
		let y_end = (y + x_offset).fract() * y_offset;

		color += uv_current.y.smoothstep(y_start, y_start + 0.05)
			* uv_current.y.smoothstep(y_end, y_end - 0.05);
	}

	vec3(noise.fit1101(), color.y, color.z).powf(2.2).extend(1.)
}
