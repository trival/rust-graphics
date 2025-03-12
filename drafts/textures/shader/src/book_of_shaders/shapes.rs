#![allow(unused_imports)]

use spirv_std::glam::{vec3, Vec2, Vec4};
use spirv_std::num_traits::Float;

use crate::utils::{flip_y, step};

pub fn rect(st: Vec2) -> Vec4 {
	let st = flip_y(st);

	// let bl = step(0.25, st.x) * step(0.25, st.y);
	// let tr = step(0.25, 1.0 - st.x) * step(0.25, 1.0 - st.y);

	// let val = bl * tr;

	// let vert = step(0.25, st.x) * step(0.25, 1.0 - st.x);
	// let horz = step(0.25, st.y) * step(0.25, 1.0 - st.y);
	let vert = (Vec2::splat(0.5) - 0.25).abs() - st.x.abs();
	let horz = (Vec2::splat(0.5) - 0.25).abs() - st.y.abs();
	let val = vert * horz;

	let color = vec3(val.x, val.y, 0.0);

	color.extend(1.0)
}
