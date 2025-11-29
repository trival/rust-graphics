use shared_nostd::aspect_preserving_uv;
use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4, vec2};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_nostd::{
	prelude::*,
	random::{hash::hash3d, simplex::simplex_noise_2d},
};

const NUM_LINES: u32 = 15;

pub fn tiled_lines(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let uv_scaled = uv * 10.;
	let uv_tile = uv_scaled.frct().fit0111();
	let idx = uv_scaled.floor();

	let mut color = Vec3::ONE;

	let noise = simplex_noise_2d(uv * vec2(5., 2.));

	for l in 0..NUM_LINES {
		for i_w in 0..3 {
			for i_h in 0..3 {
				let w = i_w as f32 - 1.;
				let h = i_h as f32 - 1.;
				let r = hash3d((idx + 10. - vec2(w, h)).extend(l as f32).to_bits());
				let mut start_x = -1. + r.x;
				let mut end_x = start_x + 0.75.lerp(1. - start_x, r.y);
				let y = r.z.fit0111() / 2.7 + noise * 0.15;

				let t = ((time * 0.2 + r.y) * (r.z + 0.1)).frct();
				if t < 0.5 {
					end_x = start_x.lerp(end_x, t * 2.);
				} else {
					start_x = start_x.lerp(end_x, (t - 0.5) * 2.);
				}

				let w_third = w * 2. / 3.;
				let h_third = h * 2. / 3.;
				let uv = uv_tile / 3. + vec2(w_third, h_third);

				let line = uv.x.smoothstep(start_x, start_x + 0.05)
					* uv.x.smoothstep(end_x, end_x - 0.05)
					* uv.y.smoothstep(y - 0.025, y - 0.02)
					* uv.y.smoothstep(y + 0.025, y + 0.02);

				// if uv_tile.x > 0.99 || uv_tile.y > 0.99 {
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
				color = color.lerp(line_color, line);
				// }
			}
		}
	}

	color.powf(2.2).extend(1.)
}
