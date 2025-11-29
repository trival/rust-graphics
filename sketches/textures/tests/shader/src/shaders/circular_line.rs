use core::f32::consts::TAU;

use shared_nostd::aspect_preserving_uv;
use spirv_std::glam::*;
use trivalibs_nostd::{prelude::*, random::simplex::simplex_noise_2d};

const TURN_COUNT: f32 = 10.0;
const BASE_RADIUS: f32 = 0.6;
const RADIUS_JITTER: f32 = 0.24;
const LINE_WIDTH: f32 = 0.085;

pub fn shader(uv: Vec2, size: UVec2, time: f32) -> Vec4 {
	let uv_current = aspect_preserving_uv(uv, size);
	let uv = uv_current * 2.0 - 1.0;

	let radius = uv.length();
	let mut angle = uv.y.atan2(uv.x);

	if angle < 0.0 {
		angle += TAU;
	}

	let angle_fraction = angle / TAU;

	// Computes the color gradient along the entire 5-turn path (red -> blue).
	let color_for_progress = |progress: f32| -> Vec3 {
		let end = Vec3::new(0.95, 0.15, 0.1);
		let start = Vec3::new(0.15, 0.3, 0.95);
		start.lerp(end, progress.clamp(0.0, 1.0))
	};

	// Computes the radius for a given turn with a stable noise wobble.
	let radius_for_turn = |phase: f32, time: f32| -> f32 {
		let noise = simplex_noise_2d(vec2(time, phase * 2.3 + 10.)) * RADIUS_JITTER;
		BASE_RADIUS + noise
	};

	// Computes intensity, color and draw height for one turn.
	let sample_turn = |turn: f32| -> (f32, Vec3) {
		let phase = angle_fraction + turn;
		let target_radius = radius_for_turn(phase, time * 0.1);
		let dist = (radius - target_radius).abs();
		let line_intensity = dist.ltf(LINE_WIDTH);
		let progress = phase / TURN_COUNT;

		(line_intensity, color_for_progress(progress))
	};

	let mut color = Vec3::ZERO;

	let mut idx = 0.;
	while idx < TURN_COUNT {
		let line = sample_turn(idx);
		color = color.lerp(line.1, line.0);
		idx += 1.;
	}

	color.powf(2.2).extend(1.0)
}
