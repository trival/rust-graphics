use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;
use trivalibs_shaders::{float_ext::FloatExt, vec_ext::VecExt};

pub fn net(uv: Vec2, _size: UVec2) -> Vec4 {
	let idx = (uv * 8.0).floor() % 2.0;
	let uv = (uv * 8.0).fract().fit0111();
	// let uv = uv.fit0111();
	let tube_x = (1.1 - uv.x.abs().max(0.0)) * 0.85;
	let tube_x = tube_x.smoothen();

	let tube_y = (1.1 - uv.y.abs().max(0.0)) * 0.85;
	let tube_y = tube_y.smoothen();

	let tube = if (idx.x + idx.y) % 2.0 == 1.0 {
		if tube_x < (tube_y * 0.2) {
			tube_y * (tube_x * 0.2 + 0.8)
			// tube_y
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
