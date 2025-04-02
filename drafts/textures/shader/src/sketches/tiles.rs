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

const NUM_LINES: u32 = 10;

pub fn tiled_lines(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let uv_scaled = uv * 10.;
	let uv = uv_scaled.fract().fit0111();
	let idx = uv_scaled.floor();

	let mut color = Vec3::ONE;

	for l in 0..NUM_LINES {
		for i_w in 0..3 {
			let w = i_w as f32 - 1.;
			let r = hash3d((idx + 10. - vec2(w, 0.)).extend(l as f32).to_bits());
			let mut start_x = -1. + r.x;
			let mut end_x = start_x + 0.75.lerp(1. - start_x, r.y);
			let y = r.z.fit0111() / 3.2;

			let t = ((time * 0.2 + r.y) * (r.z + 0.1)).fract();
			if t < 0.5 {
				end_x = start_x.lerp(end_x, t * 2.);
			} else {
				start_x = start_x.lerp(end_x, (t - 0.5) * 2.);
			}

			let w_third = w * 2. / 3.;
			let uv_tile = uv / 3. + vec2(w_third, 0.);

			let line = uv_tile.x.step(start_x)
				* end_x.step(uv_tile.x)
				* uv_tile.y.step(y - 0.01)
				* (y + 0.01).step(uv_tile.y);

			// if uv.x > 0.99 || uv.y > 0.99 {
			// 	color = vec3(0.4, 0.4, 0.4);
			// 	// Vec3::ONE
			// } else {
			let line_color = if w < -0.1 {
				// vec3(1.0, 0., 0.)
				Vec3::ZERO
			} else if w > 0.1 {
				// vec3(0., 0., 1.0)
				Vec3::ZERO
			} else {
				Vec3::ZERO
			};
			color = if line > 0. { line_color } else { color };
			// }
		}
	}

	color.extend(1.)
}
