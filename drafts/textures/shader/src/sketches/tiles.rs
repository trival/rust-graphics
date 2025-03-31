use spirv_std::glam::{vec3, UVec2, Vec2, Vec4};
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

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
