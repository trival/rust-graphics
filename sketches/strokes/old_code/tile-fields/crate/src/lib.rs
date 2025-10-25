use geom::{create_ground, create_wall};
use painting::{
	Painting, TileData, create_painting, get_painting_animated_layer, get_painting_static_layer,
};
use serde::Serialize;
use std::f32::consts::PI;
use std::vec;
use tvs_libs::rendering::buffered_geometry::BufferedGeometry;
use tvs_libs::rendering::camera::{CamProps, PerspectiveCamera};
use tvs_libs::utils::rand_utils::rand_int;
use tvs_libs::{prelude::*, setup_camera_interactions};
use wasm_bindgen::prelude::*;

pub mod geom;
pub mod painting;
pub mod utils;

#[derive(AppState, Default)]
struct State {
	paintings: Vec<Painting>,
	camera: PerspectiveCamera,
}

fn angle_radius_to_vec(angle: f32, radius: f32) -> Vec2 {
	vec2(angle.cos(), angle.sin()) * radius
}

#[derive(Serialize)]
struct PaintingData {
	width: usize,
	height: usize,
	tiles: Vec<TileData>,
	canvas_geometry: BufferedGeometry,
	mat: Mat4,
}

#[derive(Serialize)]
struct InitialData {
	paintings: Vec<PaintingData>,
	wall_geometry: BufferedGeometry,
	ground_geometry: BufferedGeometry,
	ceil_mat: Mat4,
}

#[wasm_bindgen]
pub fn get_init_data(paintings_count: usize) -> JsValue {
	let color_count = 3 + rand_int(4) as u8;
	let ps = (0..paintings_count)
		.map(|_| create_painting(rand_int(1200) + 1200, rand_int(1200) + 1200, color_count))
		.collect::<Vec<_>>();

	State::mutate(|s| {
		let camera = PerspectiveCamera::create(CamProps {
			translation: Some(vec3(0., 5., 0.)),
			rot_horizontal: Some(PI / 2.),
			..default()
		});

		s.paintings = ps.clone();
		s.camera = camera;
	});

	let paintings = ps
		.iter()
		.enumerate()
		.map(|(i, p)| {
			let angle = i as f32 / paintings_count as f32 * 2. * PI;
			let coords = angle_radius_to_vec(angle, 31.);
			PaintingData {
				width: p.width,
				height: p.height,
				tiles: get_painting_static_layer(p),
				canvas_geometry: geom::create_canvas(p.width, p.height),
				mat: Mat4::from_rotation_translation(
					Quat::from_rotation_y(-angle - PI * 0.5),
					vec3(coords.x, 0., coords.y),
				),
			}
		})
		.collect::<Vec<_>>();

	serde_wasm_bindgen::to_value(&InitialData {
		paintings,
		wall_geometry: create_wall(),
		ground_geometry: create_ground(),
		ceil_mat: Mat4::from_rotation_translation(Quat::from_rotation_x(PI), Vec3::Y * 30.),
	})
	.unwrap()
}

#[wasm_bindgen]
pub fn get_single_painting(width: usize, height: usize, color_count: u8) -> JsValue {
	let p = create_painting(width, height, color_count);

	State::mutate(|s| {
		s.paintings = vec![p.clone()];
	});

	serde_wasm_bindgen::to_value(&PaintingData {
		width: p.width,
		height: p.height,
		tiles: get_painting_static_layer(&p),
		canvas_geometry: geom::create_canvas(p.width, p.height),
		mat: Mat4::IDENTITY,
	})
	.unwrap()
}

#[wasm_bindgen]
pub fn get_painting_animation(i: usize) -> JsValue {
	let s = State::read();

	let painting = get_painting_animated_layer(&s.paintings[i]);

	serde_wasm_bindgen::to_value(&painting).unwrap()
}

#[wasm_bindgen]
pub fn get_painting_static(i: usize) -> JsValue {
	let s = State::read();

	let p = get_painting_static_layer(&s.paintings[i]);

	serde_wasm_bindgen::to_value(&p).unwrap()
}

#[derive(Serialize)]
struct FrameData {
	camera: Mat4,
	reflected_camera: Mat4,
}

#[wasm_bindgen]
pub fn get_frame_data() -> JsValue {
	let s = State::read();

	serde_wasm_bindgen::to_value(&FrameData {
		camera: s.camera.view_proj_mat(),
		reflected_camera: s.camera.reflected_cam_ground().view_proj_mat(),
	})
	.unwrap()
}

#[wasm_bindgen]
pub fn setup() {
	utils::set_panic_hook();
}

setup_camera_interactions!(State, camera);
