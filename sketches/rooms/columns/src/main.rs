use std::f32::consts::{PI, TAU};
use trivalibs::{
	common_utils::camera_controls::BasicFirstPersonCameraController,
	map,
	math::transform::Transform,
	painter::{prelude::*, utils::input_state::InputState},
	prelude::*,
	rendering::{
		BufferedGeometry,
		camera::{CamProps, PerspectiveCamera},
		mesh_geometry::{
			MeshBufferType, MeshGeometry, face_normal, face_props,
			utils::{Vert3dUv, vert_pos_uv},
		},
		scene::SceneObject,
		shapes::{cuboid::Cuboid, quad::Quad3D},
	},
};

pub fn create_plane(width: f32, height: f32, normal: Vec3, center: Vec3) -> BufferedGeometry {
	let plane: Quad3D<Vert3dUv> =
		Quad3D::from_dimensions_center_f(width, height, normal, center, vert_pos_uv).into();

	let mut geom = MeshGeometry::new();
	geom.add_face_data(&plane.to_ccw_verts(), face_normal(plane.normal));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormals)
}

pub fn create_balk_form(width: f32, height: f32, length: f32) -> BufferedGeometry {
	let bbox = Cuboid::box_at(Vec3::ZERO, width, height, length);

	let mut geom = MeshGeometry::new();

	let left = bbox.left_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.z, uvw.y)));
	geom.add_face_data(&left.to_ccw_verts(), face_props(left.normal, 2));

	let right = bbox.right_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.z, uvw.y)));
	geom.add_face_data(&right.to_ccw_verts(), face_props(right.normal, 3));

	let bottom = bbox.bottom_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.z)));
	geom.add_face_data(&bottom.to_ccw_verts(), face_props(bottom.normal, 5));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormals)
}

pub fn create_column_form(width: f32, height: f32) -> BufferedGeometry {
	let bbox = Cuboid::box_at(Vec3::ZERO, width, height, width);

	let mut geom = MeshGeometry::new();

	let front = bbox.front_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.y)));
	geom.add_face_data(&front.to_ccw_verts(), face_props(front.normal, 0));

	let back = bbox.back_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.x, uvw.y)));
	geom.add_face_data(&back.to_ccw_verts(), face_props(back.normal, 1));

	let left = bbox.left_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.z, uvw.y)));
	geom.add_face_data(&left.to_ccw_verts(), face_props(left.normal, 2));

	let right = bbox.right_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.z, uvw.y)));
	geom.add_face_data(&right.to_ccw_verts(), face_props(right.normal, 3));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormals)
}

struct App {
	cam: PerspectiveCamera,
	vp_mat: BindingBuffer<Mat4>,
	canvas: Layer,

	input: InputState,
	cam_controller: BasicFirstPersonCameraController,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p
			.shade(&[Float32x3, Float32x2, Float32x3])
			.with_bindings(&[
				BINDING_BUFFER_VERT,
				BINDING_BUFFER_VERT,
				BINDING_BUFFER_VERT,
			])
			.create();
		load_vertex_shader!(shade, p, "../shader/ground_vert.spv");
		load_fragment_shader!(shade, p, "../shader/ground_frag.spv");

		let cam = PerspectiveCamera::create(CamProps {
			fov: Some(0.6),
			translation: Some(vec3(0.0, 3.0, 15.0)),
			..default()
		});

		let to_binding = |p: &mut Painter, t: Transform| {
			let m_mat = t.model_mat();
			let n_mat = t.model_normal_mat();
			let u_m_mat = p.bind_const_mat4(m_mat);
			let u_n_mat = p.bind_const_mat3(n_mat);
			InstanceBinding {
				bindings: map! {
					0 => u_m_mat,
					1 => u_n_mat
				},
				..default()
			}
		};

		let to_instances = |p: &mut Painter, transforms: Vec<Transform>| {
			transforms.iter().map(|t| to_binding(p, *t)).collect()
		};

		let ground_form = p
			.form(&create_plane(200.0, 200.0, Vec3::Y, Vec3::ZERO))
			.create();
		let u = to_binding(p, Transform::IDENTITY).bindings;
		let ground_shape = p.shape(ground_form, shade).with_bindings(u).create();

		let col_space = 12.;
		let col_height = 40.;
		let col_width = 2.;
		let balk_height = col_width * 3.4;

		let cols_count_z_half = 8.;
		let cols_count_x_half = 4.;

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
			column_transforms.push(Transform::from_translation(vec3(
				-(cols_count_x_half - 1.) * col_space,
				col_height / 2.,
				i as f32 * col_space,
			)));
			column_transforms.push(Transform::from_translation(vec3(
				(cols_count_x_half - 1.) * col_space,
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
			column_transforms.push(Transform::from_translation(vec3(
				i as f32 * col_space,
				col_height / 2.,
				-(cols_count_z_half - 1.) * col_space,
			)));
			column_transforms.push(Transform::from_translation(vec3(
				i as f32 * col_space,
				col_height / 2.,
				(cols_count_z_half - 1.) * col_space,
			)));
		}

		let column_instances = to_instances(p, column_transforms);
		let column_shape = p
			.shape(column_form, shade)
			.with_instances(column_instances)
			.create();

		let balk_form = p
			.form(&create_balk_form(col_width, balk_height, col_space))
			.create();
		let mut balk_transforms = vec![];
		for i in -i_cols_count_x_half..=i_cols_count_x_half {
			balk_transforms.push(
				Transform::from_translation(vec3(
					i as f32 * col_space,
					col_height - balk_height / 2.,
					0.,
				))
				.with_scale(vec3(1., 1., cols_count_z_half * 2.)),
			);
		}
		for i in -i_cols_count_z_half..=i_cols_count_z_half {
			balk_transforms.push(
				Transform::from_translation(vec3(
					0.,
					col_height - balk_height / 2.0,
					i as f32 * col_space,
				))
				.with_scale(vec3(1.0, 1., cols_count_x_half * 2.))
				.with_rotation(Quat::from_rotation_y(PI / 2.)),
			);
		}
		let balk_instances = to_instances(p, balk_transforms);
		let balk_shape = p
			.shape(balk_form, shade)
			.with_instances(balk_instances)
			.create();

		let wall_form = p
			.form(&create_plane(
				col_space,
				col_height,
				Vec3::Z,
				vec3(0., col_height / 2., 0.),
			))
			.create();

		let wall_transforms = vec![
			Transform::from_xyz(0., 0., -cols_count_z_half * col_space).with_scale(vec3(
				cols_count_x_half * 2. + 2.,
				1.,
				1.,
			)),
			Transform::from_xyz(-cols_count_x_half * col_space, 0., 0.)
				.with_rotation(Quat::from_rotation_y(TAU * 1. / 4.))
				.with_scale(vec3(cols_count_z_half * 2. + 2., 1., 1.)),
			Transform::from_xyz(0., 0., cols_count_z_half * col_space)
				.with_scale(vec3(cols_count_x_half * 2. + 2., 1., 1.))
				.with_rotation(Quat::from_rotation_y(TAU * 0.5)),
			Transform::from_xyz(cols_count_x_half * col_space, 0., 0.)
				.with_rotation(Quat::from_rotation_y(TAU * 3. / 4.))
				.with_scale(vec3(cols_count_z_half * 2. + 2., 1., 1.)),
		];

		let wall_instances = to_instances(p, wall_transforms);
		let wall_shape = p
			.shape(wall_form, shade)
			.with_instances(wall_instances)
			.create();

		let vp_mat = p.bind_mat4();

		let canvas = p
			.layer()
			.with_shapes(vec![ground_shape, column_shape, balk_shape, wall_shape])
			.with_clear_color(wgpu::Color {
				r: 0.9,
				g: 0.95,
				b: 0.99,
				a: 1.0,
			})
			.with_bindings(map! {
				2 => vp_mat.binding(),
			})
			.with_multisampling()
			.with_depth_test()
			.create();

		Self {
			cam,
			canvas,
			vp_mat,
			input: default(),
			cam_controller: BasicFirstPersonCameraController::new(3.0, 4.0),
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
			remember_window_dimensions: true,
			..default()
		})
		.start();
}
