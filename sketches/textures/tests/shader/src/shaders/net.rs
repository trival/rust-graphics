use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4};
use trivalibs_nostd::prelude::*;

pub fn shader(uv: Vec2, _size: UVec2) -> Vec4 {
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
