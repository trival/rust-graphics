use crate::utils::aspect_preserving_uv;
use spirv_std::glam::{vec2, vec3, UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{
	bits::FloatBits,
	color::hsv2rgb,
	fit::Fit,
	lerp::Lerp,
	random::{
		hash::{hash2d, hash3d},
		simplex::simplex_noise_2d,
	},
	smoothstep::Smoothstep,
	step::Step,
};

const NUM_TILES: f32 = 10.;

#[derive(Copy, Clone)]
struct Tile {
	hue: f32,
	height: f32,
}

fn tile(idx: Vec2, time: f32) -> Tile {
	let r = hash2d((idx * 17.123411).to_bits());
	let hue = (r.x + time * 0.01).fract();
	// let hue = r.x;
	let height = (time * (r.x + 0.2) + r.y).cos().fit1101();

	Tile { hue, height }
}

pub fn tiled_plates(uv: Vec2, size: UVec2, t: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let uv_scaled = uv * NUM_TILES;
	let uv_tile = uv_scaled.fract() - 0.5;
	let idx = uv_scaled.floor() + 11.;

	let dir_tr = vec2(1.0, -1.0);
	let dir_tc = vec2(0.0, -1.0);
	let dir_tl = vec2(-1.0, -1.0);
	let dir_cr = vec2(1.0, 0.0);
	let dir_cl = vec2(-1.0, 0.0);
	let dir_br = vec2(1.0, 1.0);
	let dir_bc = vec2(0.0, 1.0);
	let dir_bl = vec2(-1.0, 1.0);

	let cc = tile(idx, t);

	let tr = tile(idx + dir_tr, t);
	let tc = tile(idx + dir_tc, t);
	let tl = tile(idx + dir_tl, t);
	let cr = tile(idx + dir_cr, t);
	let cl = tile(idx + dir_cl, t);
	let br = tile(idx + dir_br, t);
	let bc = tile(idx + dir_bc, t);
	let bl = tile(idx + dir_bl, t);

	let quadrant_color = |t1: Tile, t2: Tile, t3: Tile, dir1: Vec2, dir2: Vec2, dir3: Vec2| {
		let uv1 = (uv_tile - dir1) * (1. - t1.height * 0.14);
		let uv2 = (uv_tile - dir2) * (1. - t2.height * 0.14);
		let uv3 = (uv_tile - dir3) * (1. - t3.height * 0.14);

		let tiles = [cc, t1, t2, t3];
		let uvs = [uv_tile, uv1, uv2, uv3];

		let mut ground_i = 0;

		for i in 1..4 {
			if tiles[i].height > tiles[ground_i].height {
				let uv = &uvs[i];
				let d = uv.abs() - 0.4;
				let d = d.x.min(d.y);
				let square = d.smoothstep(-0.05, 0.05);
				if square > 0.5 {
					ground_i = i;
				}
			}
		}

		let ground = tiles[ground_i];

		let mut shadow = 0.;

		for i in 0..4 {
			let tile = tiles[i];
			let height = tile.height;
			if height > ground.height {
				let uv = &uvs[i];

				// smooth rect
				let smoothness = (height - ground.height) * 0.9;
				let rect = uv.abs() * 2.0;
				let e0 = Vec2::ONE + smoothness;
				let e1 = Vec2::ONE - smoothness;
				let s = rect.smoothstep(e0, e1);
				shadow += s.x * s.y;
			}
		}

		// hsv2rgb(vec3(cc.hue, 0.8, cc.height * 0.8 + 0.2))
		hsv2rgb(vec3(
			ground.hue,
			0.7 + ground.height * 0.15,
			ground.height * 0.45 + 0.45,
		))
		.lerp(Vec3::ZERO, (shadow * 0.7).clamp(0., 1.))
	};

	let color;

	// if uv_tile.y > 0.49 || uv_tile.x > 0.49 {
	// 	color = Vec3::ZERO;
	// } else
	if uv_tile.y < 0. && uv_tile.x < 0. {
		// top left
		color = quadrant_color(tl, tc, cl, dir_tl, dir_tc, dir_cl);
	} else if uv_tile.y < 0. && uv_tile.x >= 0. {
		// top right
		color = quadrant_color(tr, tc, cr, dir_tr, dir_tc, dir_cr);
	} else if uv_tile.x < 0. {
		// bottom left
		color = quadrant_color(bl, bc, cl, dir_bl, dir_bc, dir_cl);
	} else {
		// bottom right
		color = quadrant_color(br, bc, cr, dir_br, dir_bc, dir_cr);
	}

	color.extend(1.)
}

const NUM_LINES: u32 = 15;

pub fn tiled_lines(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let uv_scaled = uv * 10.;
	let uv_tile = uv_scaled.fract().fit0111();
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

				let t = ((time * 0.2 + r.y) * (r.z + 0.1)).fract();
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

	color.extend(1.)
}
