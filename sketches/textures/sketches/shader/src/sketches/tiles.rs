use shared_nostd::{
	aspect_preserving_uv,
	shapes::{rounded_rect, rounded_rect_smooth},
};
use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4, vec2, vec3};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_nostd::{color::hsv2rgb_smooth, prelude::*, random::hash::hash2d};

const NUM_TILES: f32 = 15.;

#[derive(Copy, Clone)]
struct Tile {
	hue: f32,
	lightness: f32,
	height: f32,
}

fn tile(idx: Vec2, time: f32) -> Tile {
	let r = hash2d((idx * 17.123411).to_bits());
	let hue = (r.x + time * 0.01).frct();
	// let hue = r.x;
	let height = (time * (r.x + 0.2) + r.y).cos().fit1101();
	let l = r.y * r.x * 0.4;
	let lightness = if r.y > 0.5 { 1.0 - l } else { l };

	Tile {
		hue,
		height,
		lightness,
	}
}

pub fn tiled_plates(uv: Vec2, size: UVec2, t: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let uv_scaled = uv * NUM_TILES;
	let uv_tile = uv_scaled.frct() - 0.5;
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
		let uv0 = uv_tile * (1. - cc.height * 0.14);
		let uv1 = (uv_tile - dir1) * (1. - t1.height * 0.14);
		let uv2 = (uv_tile - dir2) * (1. - t2.height * 0.14);
		let uv3 = (uv_tile - dir3) * (1. - t3.height * 0.14);

		let tiles = [cc, t1, t2, t3];
		let uvs = [uv0, uv1, uv2, uv3];

		let mut ground_i = 0;
		let mut miss = true;

		for i in 0..4 {
			if tiles[i].height >= tiles[ground_i].height {
				let uv = uvs[i];
				let square = rounded_rect(uv, Vec2::ZERO, Vec2::splat(1.0), 0.2);
				if square > 0.5 {
					ground_i = i;
					miss = false;
				}
			}
		}

		let ground = tiles[ground_i];

		let mut shadow = 0.;

		for i in 0..4 {
			let tile = tiles[i];
			let height = tile.height;
			if height > ground.height {
				let uv = uvs[i];

				// smooth rect
				let smoothness = (height - ground.height) * 0.7;
				let rect = rounded_rect_smooth(uv, Vec2::ZERO, Vec2::ONE, 0.2, smoothness);
				shadow += rect.powf(0.9);
			}
		}

		// hsv2rgb(vec3(cc.hue, 0.8, cc.height * 0.8 + 0.2))
		if miss {
			Vec3::ZERO
		} else {
			hsv2rgb_smooth(vec3(
				ground.hue,
				0.7 + ground.height * 0.15,
				(ground.height * 0.45 + 0.55) * (ground.lightness * 0.9 + 0.1),
			))
			.lerp(Vec3::ZERO, (shadow * 0.7).clamp01())
		}
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

	color.powf(1.2).extend(1.)
}
