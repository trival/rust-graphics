use spirv_std::glam::{vec2, UVec2, Vec2};

pub fn st_from_uv_size(uv: Vec2, size: &UVec2) -> Vec2 {
	let aspect = size.x as f32 / size.y as f32;
	if aspect > 1.0 {
		uv * vec2(1.0, 1.0 / aspect)
	} else {
		uv * vec2(aspect, 1.0)
	}
}

pub fn flip_y(uv: Vec2) -> Vec2 {
	vec2(uv.x, 1.0 - uv.y)
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = (x - edge0) / (edge1 - edge0);
	let t = t.clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}
