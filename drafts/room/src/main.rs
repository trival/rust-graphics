use geom::create_plane;
use input_state::{CameraController, InputState};
use trivalibs::{
	map,
	painter::prelude::*,
	prelude::*,
	rendering::camera::{CamProps, PerspectiveCamera},
};

mod geom;
mod input_state;

struct App {
	cam: PerspectiveCamera,
	vp: UniformBuffer<Mat4>,
	canvas: Layer,

	input: InputState,
	cam_controller: CameraController,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p
			.shade(&[Float32x3, Float32x3, Float32x2])
			.with_uniforms(&[UNIFORM_BUFFER_VERT])
			.create();
		load_vertex_shader!(shade, p, "../shader/ground_vert.spv");
		load_fragment_shader!(shade, p, "../shader/ground_frag.spv");

		let cam = PerspectiveCamera::create(CamProps {
			fov: Some(0.6),
			translation: Some(vec3(0.0, 3.0, 15.0)),
			// rot_horizontal: Some(PI),
			..default()
		});

		let ground_form = p
			.form(&create_plane(100.0, 100.0, Vec3::Y, Vec3::ZERO))
			.create();
		let roof_form = p
			.form(&create_plane(100.0, 100.0, -Vec3::Y, vec3(0.0, 10.0, 0.0)))
			.create();
		let wall_form = p
			.form(&create_plane(20.5, 5.0, Vec3::Z, vec3(15.0, 3.0, 0.0)))
			.create();

		let ground_shape = p.shape(ground_form, shade).create();
		let wall_shape = p.shape(wall_form, shade).create();
		let roof_shape = p.shape(roof_form, shade).create();

		let vp = p.uniform_mat4();

		let canvas = p
			.layer()
			.with_shapes(vec![ground_shape, wall_shape, roof_shape])
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_uniforms(map! {
				0 => vp.uniform(),
			})
			.with_multisampling()
			.with_depth_test()
			.create();

		Self {
			cam,
			canvas,
			vp,
			input: InputState::default(),
			cam_controller: CameraController::new(1.0, 1.0),
		}
	}

	fn resize(&mut self, _p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self
			.cam_controller
			.update_camera(&mut self.cam, &self.input, tpf);

		self.vp.update(p, self.cam.view_proj_mat());

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
