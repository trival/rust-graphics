use spirv_std::glam::{vec3, Vec2, Vec3, Vec4};
use spirv_std::num_traits::Float;

fn plot_line_1(st: Vec2) -> f32 {
	smoothstep(0.02, 0.0, (st.y - st.x).abs())
}

pub fn shaping_fns_1(st: Vec2) -> Vec4 {
	let color = Vec3::splat(st.x);
	let pct = plot_line_1(st);

	let color = (1.0 - pct) * color + pct * vec3(0.0, 1.0, 0.0);

	color.extend(1.0)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = (x - edge0) / (edge1 - edge0);
	let t = t.clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}
