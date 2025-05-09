use spirv_std::glam::{vec3, UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{
	bits::FloatBits,
	color::hsv2rgb,
	coords::PolarCoord,
	float_ext::FloatExt,
	random::{
		hash::{hash, hash21},
		simplex::simplex_noise_2d,
	},
	vec_ext::VecExt,
};

use crate::utils::aspect_preserving_uv;

pub fn net(uv: Vec2, _size: UVec2) -> Vec4 {
	let idx = (uv * 8.0).floor() % 2.0;
	let uv = (uv * 8.0).frct().fit0111();
	// let uv = uv.fit0111();
	let tube_x = (1.1 - uv.x.abs().max(0.0)) * 0.85;
	let tube_x = tube_x.smoothen();

	let tube_y = (1.1 - uv.y.abs().max(0.0)) * 0.85;
	let tube_y = tube_y.smoothen();

	let tube = if (idx.x + idx.y) % 2.0 == 1.0 {
		if tube_x < (tube_y * 0.2) {
			tube_y * (tube_x * 0.2 + 0.8)
		} else {
			tube_x * (tube_y * 0.2 + 0.8)
		}
	} else {
		if tube_y < (tube_x * 0.2) {
			tube_x * (tube_y * 0.2 + 0.8)
		} else {
			tube_y * (tube_x * 0.2 + 0.8)
		}
	};

	let color = Vec3::splat(tube);

	color.powf(2.2).extend(1.0)
}

pub fn pool_tiles(uv: Vec2, size: UVec2, t: f32) -> Vec4 {
	fn tile_color(tile_idx: Vec2) -> Vec3 {
		let rnd = hash21(tile_idx.to_bits());
		let n = simplex_noise_2d(tile_idx * 0.2).fit1101();

		let test = n * 0.7 + rnd * 0.3;
		let val = ((rnd.fit0111() * 0.7).round() + tile_idx.x + 50. * tile_idx.y) % 3.;
		let tile_rnd = hash(val as u32 + 345);

		let color = if test > 0.5 {
			hsv2rgb(vec3(0.6, 0.1, val / 2.5 + tile_rnd * 0.3))
		} else {
			hsv2rgb(vec3(tile_rnd * 0.5 + 0.15, 0.3 - tile_rnd * 0.1, 0.7))
		};
		color
	}

	let uv = aspect_preserving_uv(uv, size);

	let mut pc = PolarCoord::from_2d(uv - 1.2);
	pc.radius = pc.radius + (pc.radius * 10.1 - t * 3.).sin().powf(6.) * 0.004;
	let uv = pc.to_2d() + 1.2;

	let uv = uv * 50.0;
	let idx = uv.floor();

	let color = tile_color(idx);

	color.powf(2.2).extend(1.0)
}
