use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{bits::FloatBits, fit::Fit, lerp::Lerp, random::hash::hash3d, step::Step};

use crate::utils::aspect_preserving_uv;

const NUM_TILES: f32 = 10.;

pub fn tiled_plates(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let uv_scaled = uv * NUM_TILES;
	let uv = uv_scaled.fract();
	let idx = uv_scaled.floor();
	let color = vec3(
		uv.x,
		uv.y,
		(time + idx.x / (2. * NUM_TILES) + idx.y / (2.0 * NUM_TILES)).fract(),
	);

	color.extend(1.)
}

const NUM_LINES: u32 = 4;
pub fn tiled_lines(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let uv_scaled = uv * 10.;
	let uv = uv_scaled.fract().fit0111();
	let idx = uv_scaled.floor();

	let mut line = 0.;

	for l in 0..NUM_LINES {
		for i_w in 0..3 {
			let w = i_w as f32 - 1.;
			let r = hash3d((idx + 1.0 - vec2(w, 0.)).extend(l as f32).to_bits());
			let start_x = -1. + r.x;
			let end_x = start_x + 1.0.lerp(1. - start_x, r.y);
			let y = r.z.fit0111();

			let w = w / 3.;
			let uv = uv / 3. + vec2(w, 0.);
			line += uv.x.step(start_x) * end_x.step(uv.x) * uv.y.step(y - 0.02) * (y + 0.02).step(uv.y);
		}
	}

	let color = if uv.x > 0.99 || uv.y > 0.99 {
		vec3(1., 0., 0.)
		// Vec3::ONE
	} else {
		Vec3::ONE.lerp(Vec3::ZERO, line)
	};

	color.extend(1.)
}
