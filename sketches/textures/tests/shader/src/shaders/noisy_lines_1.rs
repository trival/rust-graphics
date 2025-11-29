use shared_nostd::aspect_preserving_uv;
use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4, vec2};
use trivalibs_nostd::{prelude::*, random::simplex::simplex_noise_3d};

const LINE_COUNT: f32 = 30.;

pub fn shader(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv_current = aspect_preserving_uv(uv, size);

	let noise =
		simplex_noise_3d((uv_current * vec2(2.5, 1.5) - vec2(0., time * 0.6)).extend(time * 0.2));

	let x_offset = 1.0 / LINE_COUNT;
	let curr_x = uv_current.x;
	let next_x = curr_x + x_offset;
	let prev_x = curr_x - x_offset;

	let curr_x_pos = ((curr_x * LINE_COUNT).frct() - 0.5) / 3.;
	let prev_x_pos = ((prev_x * LINE_COUNT).frct() - 0.5) / 3. + 1. / 3.;
	let next_x_pos = ((next_x * LINE_COUNT).frct() - 0.5) / 3. - 1. / 3.;

	let get_line = |x: f32| x.smoothstep(-0.06, -0.04) * x.smoothstep(0.06, 0.04);

	let offset = 0.45 * noise;

	let line_curr = get_line(curr_x_pos + offset);
	let line_prev = get_line(prev_x_pos + offset);
	let line_next = get_line(next_x_pos + offset);

	let bg = Vec3::splat((noise + 1.0) / 4.0);

	let color = if (uv_current.x * LINE_COUNT).frct() < 0.02 {
		Vec3::ZERO
	} else if line_curr > 0.0 {
		bg.lerp(Vec3::new(1.0, 1.0, 1.0), line_curr)
	} else if line_next > 0.0 {
		bg.lerp(Vec3::new(0.0, 1.0, 1.0), line_next)
	} else if line_prev > 0.0 {
		bg.lerp(Vec3::new(0.0, 1.0, 0.0), line_prev)
	} else {
		bg
	};

	color.extend(1.0)
}
