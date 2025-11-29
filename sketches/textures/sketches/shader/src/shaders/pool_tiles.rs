use shared_nostd::aspect_preserving_uv;
use spirv_std::glam::{Mat2, UVec2, Vec2, Vec4, vec2, vec3};
use trivalibs_nostd::{
	color::hsv2rgb,
	coords::PolarCoord,
	prelude::*,
	random::{
		hash::{hash, hash21},
		simplex::simplex_noise_2d,
	},
};

pub fn shader(uv: Vec2, size: UVec2, t: f32) -> Vec4 {
	let uv = aspect_preserving_uv(uv, size);

	let drop_center = vec2(-1.2, -1.3);
	let mut pc = PolarCoord::from_2d(uv - drop_center);
	pc.radius = pc.radius + (pc.radius * 10.1 - t * 3.).sin().powf(6.) * 0.003;
	let uv = pc.to_2d() + drop_center;

	let drop_center = vec2(2.2, 0.3);
	let mut pc = PolarCoord::from_2d(uv - drop_center);
	pc.radius = pc.radius + (pc.radius * 8.5 - t * 1.8).sin().powf(8.) * 0.0055;
	let uv = pc.to_2d() + drop_center;

	let uv = uv - 0.5;

	let tile_scale = 50. * 1.0.lerp(0.6, uv.y) * 1.0.lerp(0.85, uv.x);
	let uv = uv * tile_scale;

	let n = simplex_noise_2d(Vec2::splat(t * 0.006)) * 2.;
	let mat = Mat2::from_angle(n);
	let uv = mat * uv;

	let nx = simplex_noise_2d(Vec2::splat(t * 0.01 - 100.));
	let ny = simplex_noise_2d(Vec2::splat(t * 0.01 - 200.));
	let offset = vec2(nx, ny) * 33.;
	let uv = uv + offset;

	let idx = uv.floor();

	let rnd = hash21(idx.to_bits());
	let n = simplex_noise_2d(idx * 0.2).fit1101();

	let test = n * 0.7 + rnd * 0.3;
	let val = ((rnd.fit0111() * 0.7).round() + idx.x + 50. * idx.y).rem(3.);
	let tile_rnd = hash(val as u32 + 345);

	let color = if test > 0.5 {
		hsv2rgb(vec3(0.6, 0.1, val / 2.5 + tile_rnd * 0.3))
	} else {
		hsv2rgb(vec3(tile_rnd * 0.5 + 0.15, 0.3 - tile_rnd * 0.1, 0.7))
	};

	color.powf(2.2).extend(1.0)
}
