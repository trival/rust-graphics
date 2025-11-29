use shared_nostd::aspect_preserving_uv;
use spirv_std::glam::{UVec2, Vec2, Vec3, Vec4, vec2, vec3};
use trivalibs_nostd::{
	prelude::*,
	random::{
		hash::hash,
		simplex::{simplex_noise_2d, simplex_noise_3d},
	},
};

const LINE_COUNT: f32 = 20.0;

pub fn shader(uv: Vec2, _size: UVec2, time: f32) -> Vec4 {
	let line_segment = (uv.x * LINE_COUNT).floor();
	let line_x = (uv.x * LINE_COUNT).frct().fit0111();

	// Color generation closure
	let color = |segment: f32| {
		vec3(
			hash((segment * 3.0 * LINE_COUNT) as u32),
			hash((segment * 7.0 * LINE_COUNT) as u32),
			hash((segment * 11.0 * LINE_COUNT) as u32),
		)
	};

	// Height/depth calculation closure for z-ordering
	let compute_height = |segment: f32| -> f32 { hash((segment * LINE_COUNT) as u32) };

	// Complete line computation closure - returns (intensity, color, height)
	let compute_line = |segment_offset: f32, line_x_offset: f32| -> (f32, Vec3, f32) {
		let segment = line_segment + segment_offset;
		let x = line_x + line_x_offset;
		let noise = simplex_noise_3d(vec3(segment, uv.y + time * 0.1, time * 0.07)) * 1.3;
		let x_bent = x + noise;
		let line_intensity = x_bent.abs().smoothstep(0.7, 0.6);
		let line_color = color(segment);
		let height = compute_height(segment);
		(line_intensity, line_color, height)
	};

	// Pass closure - computes and sorts 3 lines for a given segment offset
	let compute_pass = |pass_segment_offset: f32| -> [(f32, Vec3, f32); 3] {
		// Generate all three line variants with pass offset
		let curr = compute_line(pass_segment_offset + 0.0, 0.0);
		let prev = compute_line(pass_segment_offset - 1.0, 2.0);
		let next = compute_line(pass_segment_offset + 1.0, -2.0);

		// Sort by height using manual swapping (bubble sort for 3 elements)
		let mut lines = [prev, curr, next];

		// Pass 1
		if lines[0].2 > lines[1].2 {
			let temp = lines[0];
			lines[0] = lines[1];
			lines[1] = temp;
		}
		if lines[1].2 > lines[2].2 {
			let temp = lines[1];
			lines[1] = lines[2];
			lines[2] = temp;
		}
		// Pass 2
		if lines[0].2 > lines[1].2 {
			let temp = lines[0];
			lines[0] = lines[1];
			lines[1] = temp;
		}

		lines
	};

	// Blend closure - blends 3 sorted lines onto a base color
	let blend_pass = |base_color: Vec3, lines: [(f32, Vec3, f32); 3]| -> Vec3 {
		base_color
			.lerp(lines[0].1, lines[0].0)
			.lerp(lines[1].1, lines[1].0)
			.lerp(lines[2].1, lines[2].0)
	};

	// Execute 3 passes, each layering on top of the previous
	let col_bg = Vec3::ONE;
	let col = blend_pass(col_bg, compute_pass(0.0)); // Pass 1: Base layer
	let col = blend_pass(col, compute_pass(100.0)); // Pass 2: Middle layer
	let col = blend_pass(col, compute_pass(200.0)); // Pass 3: Top layer

	col.powf(2.2).extend(1.0)
}

pub fn noisy_lines_3(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv_current = aspect_preserving_uv(uv, size);

	let uv = uv_current * 2.0 - 1.0;

	let bend_noise = simplex_noise_2d(vec2(uv.y * 0.6, time * 0.5)).fit1101();

	let texture_noise =
		(simplex_noise_2d((uv_current + vec2(bend_noise / (15.0 / 2.0), 0.0)) * vec2(92.0, 02.0))
			+ simplex_noise_2d(
				(uv_current + vec2(bend_noise / (15.0 / 2.0), 0.0)) * vec2(92.0, 2.0) * 2.0 + 120.,
			) * 0.5)
			/ 1.5;

	let grid = (uv_current * 340.).sin().fit1101() * 0.5 + 0.5;
	let bg_texture_noise =
		((simplex_noise_2d(uv_current * 200.) + simplex_noise_2d(uv_current * 400. + 12.) * 0.5) / 1.5)
			* grid.x
			* grid.y;

	let y = uv.y * 0.96;
	let caps = y.abs().step_fn(1.0, 0.75, |t| t.powf(0.25)) + 0.01;

	let x = uv.x * 15.0 / caps + bend_noise * 3.5 / caps;

	let line = x.abs().smoothstep(1.0, 0.8)
		* y.abs().smoothstep(1.0, 0.95)
		* ((texture_noise
			.fit1101()
			.powf(0.2 + y.abs().step_fn(0.88, 1.0, |t| t.powf(3.)) * 9.)
			+ bg_texture_noise * 0.1)
			/ 1.1)
			.clamp01()
			.powf(1.5);

	let col_bg = Vec3::ONE;
	let col_line = vec3(0.2, 0.2, 0.2);

	let col = col_bg.lerp(col_line, line);

	col.powf(2.2).extend(1.0)
}
