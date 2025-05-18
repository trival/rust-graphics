use std::f32::consts::PI;

use trivalibs::{
	common_utils::camera_controls::BasicFirstPersonCameraController,
	map,
	math::transform::Transform,
	painter::{prelude::*, utils::input_state::InputState},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		mesh_geometry::{
			face_normal, face_section,
			utils::{vert_pos_uv, Vert3dUv},
			FaceDataProps, MeshBufferType, MeshGeometry,
		},
		scene::SceneObject,
		shapes::{cuboid::Cuboid, quad::Quad3D},
		BufferedGeometry,
	},
};

pub fn create_plane(width: f32, height: f32, normal: Vec3, center: Vec3) -> BufferedGeometry {
	let plane: Quad3D<Vert3dUv> =
		Quad3D::from_dimensions_center_f(width, height, normal, center, vert_pos_uv).into();

	let mut geom = MeshGeometry::new();
	// geom.add_face4_data(plane.to_ccw_verts(), face_normal(plane.normal));
	geom.add_face4(plane.to_ccw_verts());

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}

pub fn create_balk_form(width: f32, height: f32, length: f32) -> BufferedGeometry {
	let bbox = Cuboid::box_at(Vec3::ZERO, width, height, length);

	let mut geom = MeshGeometry::new();

	// let face_data = |normal: Vec3, section: usize| FaceDataProps {
	// 	normal: Some(normal),
	// 	section: Some(section),
	// 	data: None,
	// };

	let left = bbox.left_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.z, uvw.y)));
	// geom.add_face4_data(left.to_ccw_verts(), face_data(left.normal, 2));
	geom.add_face4_data(left.to_ccw_verts(), face_section(2));

	let right = bbox.right_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.z, uvw.y)));
	// geom.add_face4_data(right.to_ccw_verts(), face_data(right.normal, 3));
	geom.add_face4_data(right.to_ccw_verts(), face_section(3));

	let bottom = bbox.bottom_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.z)));
	// geom.add_face4_data(bottom.to_ccw_verts(), face_data(bottom.normal, 5));
	geom.add_face4_data(bottom.to_ccw_verts(), face_section(5));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}

pub fn create_column_form(width: f32, height: f32) -> BufferedGeometry {
	let bbox = Cuboid::box_at(Vec3::ZERO, width, height, width);

	let mut geom = MeshGeometry::new();

	let face_data = |normal: Vec3, section: usize| FaceDataProps {
		normal: Some(normal),
		section: Some(section),
		data: None,
	};

	let front = bbox.front_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.y)));
	geom.add_face4_data(front.to_ccw_verts(), face_data(front.normal, 0));

	let back = bbox.back_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.x, uvw.y)));
	geom.add_face4_data(back.to_ccw_verts(), face_data(back.normal, 1));

	let left = bbox.left_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.z, uvw.y)));
	geom.add_face4_data(left.to_ccw_verts(), face_data(left.normal, 2));

	let right = bbox.right_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.z, uvw.y)));
	geom.add_face4_data(right.to_ccw_verts(), face_data(right.normal, 3));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}

struct App {
	cam: PerspectiveCamera,
	vp_mat: UniformBuffer<Mat4>,
	canvas: Layer,

	input: InputState,
	cam_controller: BasicFirstPersonCameraController,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p
			.shade(&[Float32x3, Float32x3, Float32x2])
			.with_uniforms(&[
				UNIFORM_BUFFER_VERT,
				UNIFORM_BUFFER_VERT,
				UNIFORM_BUFFER_VERT,
			])
			.create();
		load_vertex_shader!(shade, p, "../shader/ground_vert.spv");
		load_fragment_shader!(shade, p, "../shader/ground_frag.spv");

		let cam = PerspectiveCamera::create(CamProps {
			fov: Some(0.6),
			translation: Some(vec3(0.0, 3.0, 15.0)),
			..default()
		});

		let ground_form = p
			.form(&create_plane(200.0, 200.0, Vec3::Y, Vec3::ZERO))
			.create();
		let g_m_mat = p.uniform_const_mat4(Mat4::IDENTITY);
		let g_n_mat = p.uniform_const_mat3(Mat3::IDENTITY);
		let ground_shape = p
			.shape(ground_form, shade)
			.with_uniforms(map! {
				0 => g_m_mat,
				1 => g_n_mat
			})
			.create();

		let col_space = 12.;
		let col_height = 40.;
		let col_width = 2.;

		let cols_count_z_half = 6.;
		let cols_count_x_half = 3.;

		let i_cols_count_z_half = cols_count_z_half as i32;
		let i_cols_count_x_half = cols_count_x_half as i32;

		let column_form = p.form(&create_column_form(col_width, col_height)).create();
		let mut column_transforms = vec![];
		for i in -i_cols_count_z_half..=i_cols_count_z_half {
			column_transforms.push(Transform::from_translation(vec3(
				-cols_count_x_half * col_space,
				col_height / 2.,
				i as f32 * col_space,
			)));
			column_transforms.push(Transform::from_translation(vec3(
				cols_count_x_half * col_space,
				col_height / 2.,
				i as f32 * col_space,
			)));
		}

		for i in -i_cols_count_x_half..=i_cols_count_x_half {
			column_transforms.push(Transform::from_translation(vec3(
				i as f32 * col_space,
				col_height / 2.,
				-cols_count_z_half * col_space,
			)));
			column_transforms.push(Transform::from_translation(vec3(
				i as f32 * col_space,
				col_height / 2.,
				cols_count_z_half * col_space,
			)));
		}

		let to_instances = |p: &mut Painter, transforms: Vec<Transform>| {
			transforms
				.iter()
				.map(|t| {
					let m_mat = t.model_mat();
					let n_mat = t.model_normal_mat();
					let u_m_mat = p.uniform_const_mat4(m_mat);
					let u_n_mat = p.uniform_const_mat3(n_mat);
					InstanceUniforms {
						uniforms: map! {
							0 => u_m_mat,
							1 => u_n_mat
						},
						..default()
					}
				})
				.collect()
		};

		let column_instances = to_instances(p, column_transforms);
		let column_shape = p
			.shape(column_form, shade)
			.with_instances(column_instances)
			.create();

		let balk_form = p
			.form(&create_balk_form(col_width, col_width * 3., col_space))
			.create();
		let mut balk_transforms = vec![];
		for i in -i_cols_count_x_half..=i_cols_count_x_half {
			balk_transforms.push(
				Transform::from_translation(vec3(i as f32 * col_space, col_height, 0.)).with_scale(vec3(
					1.,
					1.,
					cols_count_z_half * 2.,
				)),
			);
		}
		for i in -i_cols_count_z_half..=i_cols_count_z_half {
			balk_transforms.push(
				Transform::from_translation(vec3(0., col_height, i as f32 * col_space))
					.with_scale(vec3(1.0, 1., cols_count_x_half * 2.))
					.with_rotation(Quat::from_rotation_y(PI / 2.)),
			);
		}
		let balk_instances = to_instances(p, balk_transforms);
		let balk_shape = p
			.shape(balk_form, shade)
			.with_instances(balk_instances)
			.create();

		let vp_mat = p.uniform_mat4();

		let canvas = p
			.layer()
			.with_shapes(vec![ground_shape, column_shape, balk_shape])
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_uniforms(map! {
				2 => vp_mat.uniform(),
			})
			.with_multisampling()
			.with_depth_test()
			.create();

		Self {
			cam,
			canvas,
			vp_mat,
			input: default(),
			cam_controller: BasicFirstPersonCameraController::new(1.0, 3.0),
		}
	}

	fn resize(&mut self, _p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
		self.cam_controller.set_screen_size(width, height);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self
			.cam_controller
			.update_camera(&mut self.cam, &self.input, tpf);

		self.vp_mat.update(p, self.cam.view_proj_mat());

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		p.paint_and_show(self.canvas)
	}

	fn event(&mut self, e: Event<()>, _p: &mut Painter) {
		self.input.process_event(e);
	}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: true,
			keep_window_dimensions: true,
			..default()
		})
		.start();
}
