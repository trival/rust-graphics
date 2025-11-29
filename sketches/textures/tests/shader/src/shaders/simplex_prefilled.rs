use spirv_std::{
	Image, Sampler,
	glam::{UVec2, Vec2, Vec4, vec2},
};

pub fn shader(
	uv: Vec2,
	tex: &Image!(2D, type=f32, sampled),
	sampler: &Sampler,
	size: &UVec2,
) -> Vec4 {
	let aspect = size.x as f32 / size.y as f32;
	let noise = tex.sample(*sampler, vec2(uv.x * aspect * 3., uv.y * 3.));
	let val = (noise.x + noise.y * 0.5 + noise.z * 0.25 + noise.w * 0.125) / 1.875;
	Vec4::new(val, val, val, 1.0)
}
