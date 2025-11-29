use crate::utils;
use shared_nostd::flip_y;
use spirv_std::glam::{Vec2, Vec3, Vec4, vec2, vec3};

pub fn shader(st: Vec2) -> Vec4 {
	let st = flip_y(st);

	let size = vec2(0.8, 0.65);

	let center1 = vec2(0.5, 0.50);
	let rec1 = utils::rect(size, center1, st);

	let center2 = vec2(0.5, 0.485);
	let rec2 = utils::rect_smooth(size, center2, st, 0.12);

	let color1 = vec3(1.0, 1.0, 0.0);
	let color2 = vec3(0.3, 0.3, 0.1);

	let bg_color = Vec3::splat(1.0);
	bg_color.lerp(color2, rec2).lerp(color1, rec1).extend(1.0)
}
