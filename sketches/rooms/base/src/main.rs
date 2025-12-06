use trivalibs::rendering::{
	mesh_geometry::{MeshBufferType, MeshGeometry},
	shapes::cuboid::Cuboid,
};
use trivalibs::{
	common_utils::camera_controls::BasicFirstPersonCameraController,
	map,
	painter::{prelude::*, utils::input_state::InputState},
	prelude::*,
	rendering::camera::{CamProps, PerspectiveCamera},
};

const ROOM_HEIGHT: f32 = 5.5;
const ROOM_DEPTH: f32 = 10.0;
const ROOM_WIDTH: f32 = 6.5;

struct App {
	cam: PerspectiveCamera,
	vp_mat: BindingBuffer<Mat4>,
	canvas: Layer,

	input: InputState,
	cam_controller: BasicFirstPersonCameraController,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let room_cube = Cuboid::box_at(
			vec3(0., ROOM_HEIGHT / 2., 0.),
			ROOM_WIDTH,
			ROOM_HEIGHT,
			ROOM_DEPTH,
		);

		let ceil = MeshGeometry::from_face(room_cube.top_face().to_cw_verts())
			.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormal);
		let floor = MeshGeometry::from_face(room_cube.bottom_face().to_cw_verts())
			.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormal);
		let walls = [
			room_cube.front_face(),
			room_cube.right_face(),
			room_cube.back_face(),
			room_cube.left_face(),
		]
		.iter()
		.map(|face| {
			MeshGeometry::from_face(face.to_cw_verts())
				.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormal)
		})
		.collect::<Vec<_>>();

		let floor_form = p.form(&floor).create();
		let ceil_form = p.form(&ceil).create();
		let walls_form = p.form_builder().with_buffers(&walls).create();

		let pre_render_shade = p.shade(&[Float32x3, Float32x2, Float32x3]).create();
		load_vertex_shader!(pre_render_shade, p, "../shader/wall_pre_render_vert.spv");
		load_fragment_shader!(pre_render_shade, p, "../shader/wall_pre_render_frag.spv");

		let wall_render_shade = p
			.shade(&[Float32x3, Float32x2, Float32x3])
			.with_bindings(&[BINDING_BUFFER_VERT, BINDING_SAMPLER_FRAG])
			.with_layers(&[BINDING_LAYER_FRAG])
			.create();
		load_vertex_shader!(wall_render_shade, p, "../shader/wall_render_vert.spv");
		load_fragment_shader!(wall_render_shade, p, "../shader/wall_render_frag.spv");

		let cam = PerspectiveCamera::create(CamProps {
			fov: Some(0.6),
			translation: Some(vec3(0.0, 3.0, 15.0)),
			// rot_horizontal: Some(PI),
			..default()
		});

		let floor_shape = p.shape(floor_form, wall_render_shade).create();
		let wall_shape = p.shape(walls_form, wall_render_shade).create();
		let ceil_shape = p.shape(ceil_form, wall_render_shade).create();

		let vp_mat = p.bind_mat4();
		let sampler = p
			.sampler()
			.with_filters(wgpu::FilterMode::Linear)
			.with_mipmap_filter(wgpu::FilterMode::Linear)
			.create();

		let canvas = p
			.layer()
			.with_shapes(vec![floor_shape, wall_shape, ceil_shape])
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_bindings(map! {
				0 => vp_mat.binding(),
				1 => sampler.binding()
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

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		self
			.cam_controller
			.update_camera(&mut self.cam, &self.input, tpf);

		self.vp_mat.update(p, self.cam.view_proj_mat());

		p.paint_and_show(self.canvas);
		// p.show(self.grid_col_tex);

		p.request_next_frame();
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
