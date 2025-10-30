use trivalibs::{
	prelude::*,
	rendering::line_2d::Line,
	utils::rand_utils::{Pick, rand_bool, rand_f32, rand_usize},
};

use trivalibs_shaders::{color::hsv2rgb, float_ext::FloatExt};

#[derive(Clone, Copy)]
pub struct Color {
	pub hue: f32,
	pub lightness: f32,
}

#[derive(Clone, Copy)]
pub struct Tile {
	pub top: f32,
	pub left: f32,
	pub width: f32,
	pub height: f32,
	pub color: Color,
}

fn subdivide_tile<F: Fn() -> Color>(
	tile: Tile,
	min_size: f32,
	max_splits: usize,
	split_variance: f32,
	split_direction_variance: f32,
	get_color: F,
) -> Vec<Tile> {
	if tile.width <= min_size || tile.height <= min_size {
		return vec![tile];
	}

	let divide_horizontally =
		tile.width / tile.height + (rand_f32() * 2.0 - 1.0) * split_direction_variance > 1.;

	let tile_length = if divide_horizontally {
		tile.width
	} else {
		tile.height
	};

	let tile_count = usize::min(max_splits, (tile_length / min_size).floor() as usize);
	if tile_count < 2 {
		return vec![tile];
	}

	let mut tiles = vec![];

	let mut split_ratios = vec![0.];
	split_ratios.append(
		&mut (1..tile_count)
			.map(|i| ((rand_f32() * 2.0 - 1.0) * 0.5 * split_variance + i as f32) / tile_count as f32)
			.collect(),
	);
	split_ratios.push(1.);

	for ps in split_ratios.windows(2) {
		let l1 = ps[0];
		let l2 = ps[1];
		let start = l1 * tile_length;
		let length = (l2 - l1) * tile_length;
		let new_tile = if divide_horizontally {
			Tile {
				left: start + tile.left,
				width: length,
				top: tile.top,
				height: tile.height,
				color: get_color(),
			}
		} else {
			Tile {
				left: tile.left,
				width: tile.width,
				top: start + tile.top,
				height: length,
				color: get_color(),
			}
		};
		tiles.push(new_tile);
	}

	tiles
}

#[derive(Clone)]
pub struct Painting {
	pub width: u32,
	pub height: u32,
	pub tiles: Vec<Tile>,
	pub brush_size: f32,
}

fn random_split(v: Vec<f32>) -> Vec<f32> {
	let idx = rand_usize(v.len() - 1);
	let item_i = v[idx];
	let item_ii = v[idx + 1];

	let mut res = vec![];
	for i in 0..=idx {
		res.push(v[i]);
		res.push(Lerp::lerp(item_i, item_ii, rand_f32()));
	}
	for i in (idx + 1)..v.len() {
		res.push(v[i]);
	}

	res
}

pub fn create_painting(width: u32, height: u32, color_count: u8) -> Painting {
	let mut hues: Vec<f32> = vec![0., 1.];
	for _ in 0..color_count - 1 {
		hues = random_split(hues);
	}
	hues.pop();

	let hue_shift = rand_f32();
	let colors = hues
		.into_iter()
		.map(|h| {
			let hue = h + hue_shift;
			Color {
				hue: hue - hue.floor(),
				lightness: (rand_f32() + rand_f32()) * 0.4 + 0.1,
			}
		})
		.collect::<Vec<_>>();

	let get_color = || colors.pick().clone();

	let brush_size = height as f32 / 50.0;

	let first_tile = Tile {
		top: 0.,
		left: 0.,
		width: width as f32,
		height: height as f32,
		color: get_color(),
	};

	let mut tiles = vec![first_tile];

	let subdivide_count = rand_usize(4) + 1;

	for _ in 0..subdivide_count {
		let mut new_tiles = vec![];
		for tile in tiles {
			let max_splits = rand_usize(3) + 2;
			new_tiles.append(&mut subdivide_tile(
				tile,
				brush_size * 3.,
				max_splits,
				0.5,
				0.5,
				get_color,
			));
		}
		tiles = new_tiles;
	}

	Painting {
		brush_size,
		tiles,
		width,
		height,
	}
}

fn get_line_edges(tile: &Tile, brush_size: f32) -> (Vec<Vec2>, bool) {
	let steps = ((tile.height * 1.3) / brush_size).floor().max(4.);
	let step = tile.height / steps;

	let mut is_left = rand_bool();

	let delta = || step * 0.2 * (rand_f32() * 2.0 - 1.0);

	let mut points = Vec::new();

	let point_w_offset = step * 0.06;

	points.push(vec2(
		if is_left {
			tile.left - point_w_offset
		} else {
			tile.left + tile.width + point_w_offset
		} + delta() * f32::max(tile.width / (brush_size * 3.), 2.),
		tile.top + step * 1.25 + delta(),
	));
	is_left = !is_left;

	for i in 1..(steps * 2. - 1.) as usize {
		points.push(vec2(
			if is_left {
				tile.left - point_w_offset
			} else {
				tile.left + tile.width + point_w_offset
			} + delta() * f32::max(tile.width / (brush_size * 3.), 2.),
			tile.top + step * i as f32 * 0.49 + step * 0.75 + delta(),
		));
		is_left = !is_left;
	}

	points.push(vec2(
		if is_left {
			tile.left - point_w_offset
		} else {
			tile.left + tile.width + point_w_offset
		} + delta() * f32::max(tile.width / (brush_size * 3.), 2.),
		tile.top + step * (steps - 1.) + delta(),
	));

	(points, is_left)
}

fn make_curve(width: f32, brush_size: f32, p1: Vec2, p2: Vec2, reverse: bool) -> Vec<Vec2> {
	let normal_scale_factor = (brush_size / 6.) * (width / brush_size).min(15.);
	let line = p2 - p1;
	let steps = ((line.length() / 35.).floor() as usize).max(8);
	let normal = if reverse {
		vec2(-line.y, line.x).normalize()
	} else {
		vec2(line.y, -line.x).normalize()
	};
	let p3 = p1 + line * 0.5 + normal * (rand_f32() - 0.6) * normal_scale_factor;
	let p4 = p1 + line * 0.5 + normal * (rand_f32() - 0.6) * normal_scale_factor;
	(0..=steps)
		.map(|t| {
			let t = t as f32 / steps as f32;
			Vec2::cubic_bezier(t, p1, p3, p4, p2)
		})
		.collect()
}

pub struct TileStrokes {
	pub lines: Vec<Line>,
	pub color: Vec3,
}

fn calculate_color(color: Color) -> Vec3 {
	hsv2rgb(vec3(
		(color.hue + rand_normal_01() * 0.1).fract(),
		((rand_f32() + rand_f32()) * 0.5).powf(1.5),
		(color.lightness + rand_normal_11() * 0.4).clamp01(),
	))
}

pub fn generate_tile_strokes(painting: &Painting) -> Vec<TileStrokes> {
	let mut result = vec![];

	let mut shuffled = painting.tiles.clone();
	shuffled.shuffle(&mut rand::rng());

	for tile in shuffled.iter() {
		let brush_size = painting
			.brush_size
			.max(tile.height / 10.)
			.min(painting.brush_size * 3.);

		let (points, mut is_left) = get_line_edges(tile, brush_size);

		let mut total_length = 0.0;
		let mut lines = vec![];

		for ps in points.windows(2) {
			let p1 = ps[0];
			let p2 = ps[1];
			let curve_points = make_curve(tile.width, brush_size, p1, p2, is_left);
			is_left = !is_left;

			let mut line = Line::new_offset(brush_size, total_length);

			for point in curve_points {
				line.add(point);
			}

			let line_length = line.line_length();
			total_length += line_length;

			lines.push(line);
		}

		result.push(TileStrokes {
			lines,
			color: calculate_color(tile.color),
		});
	}

	result
}
