use spirv_std::glam::{vec3, UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{
	bits::FloatBits,
	color::{hsv2rgb, hsv2rgb_smooth},
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

		let val = n * 0.7 + rnd * 0.3;

		let color = if val > 0.5 {
			let val = (rnd * 3.).floor();
			let n = hash(val as u32 + 123);
			hsv2rgb_smooth(vec3(0.6, 0.1, val / 3. + n * 0.3))
		} else {
			let val = (val * 2. * 3.).floor();
			let n = hash(val as u32 + 345);
			hsv2rgb(vec3(0.43 + n * 0.3, 0.4 - n * 0.1, 0.7))
		};
		color
	}

	let uv = aspect_preserving_uv(uv, size);

	let mut pc = PolarCoord::from_2d(uv - 0.5);
	pc.radius = pc.radius - (pc.radius * 10.1 - t * 3.).cos().powf(8.) * 0.01;
	let uv = pc.to_2d();

	let uv = uv * 30.0;
	let idx = (uv).floor();

	let color = tile_color(idx);

	color.powf(2.2).extend(1.0)
}
