use geom::create_plane;
use trivalibs::{
	common_utils::camera_controls::BasicFirstPersonCameraController,
	map,
	painter::{prelude::*, utils::input_state::InputState},
	prelude::*,
	rendering::camera::{CamProps, PerspectiveCamera},
};

use crate::geom::{create_grid_columns_form, create_grid_rows_form, GridProps};

mod geom;

struct App {
	cam: PerspectiveCamera,
	vp_mat: BindingBuffer<Mat4>,
	grid_row_tex: Layer,
	grid_col_tex: Layer,
	canvas: Layer,

	input: InputState,
	cam_controller: BasicFirstPersonCameraController,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let pre_render_shade = p.shade(&[Float32x3, Float32x2, Float32x3]).create();
		load_vertex_shader!(pre_render_shade, p, "../shader/wall_pre_render_vert.spv");
		load_fragment_shader!(pre_render_shade, p, "../shader/wall_pre_render_frag.spv");

		let grid_size = (20., 30.);

		let grid_row = create_grid_rows_form(GridProps {
			grid_width: grid_size.0,
			grid_height: grid_size.1,
			count: (grid_size.0 / grid_size.1 * 30.).floor() as usize,
			strip_height: 1.5,
			strip_width: 0.2,
			center: vec3(0., 15., 0.),
		});

		let grid_row_form = p.form(&grid_row.form).create();

		let grid_col = create_grid_columns_form(GridProps {
			grid_width: grid_size.0,
			grid_height: grid_size.1,
			count: (grid_size.1 / grid_size.0 * 30.).floor() as usize,
			strip_height: 1.5,
			strip_width: 0.2,
			center: vec3(0., 15., 0.),
		});

		let grid_col_form = p.form(&grid_col.form).create();

		let grid_row_tex_shape = p.shape(grid_row_form, pre_render_shade).create();
		let grid_col_tex_shape = p.shape(grid_col_form, pre_render_shade).create();

		let grid_row_tex = p
			.layer()
			.with_size(
				(grid_row.texture_size.0 * 50.).floor() as u32,
				(grid_row.texture_size.1 * 50.).floor() as u32,
			)
			.with_shape(grid_row_tex_shape)
			.create_and_init();

		let _ = p.paint(grid_row_tex);

		let grid_col_tex = p
			.layer()
			.with_size(
				(grid_col.texture_size.0 * 50.).floor() as u32,
				(grid_col.texture_size.1 * 50.).floor() as u32,
			)
			.with_shape(grid_col_tex_shape)
			.create_and_init();

		let _ = p.paint(grid_col_tex);

		let wall_render_shade = p
			.shade(&[Float32x3, Float32x2, Float32x3])
			.with_bindings(&[BINDING_BUFFER_VERT])
			.create();
		load_vertex_shader!(wall_render_shade, p, "../shader/wall_render_vert.spv");
		load_fragment_shader!(wall_render_shade, p, "../shader/wall_render_frag.spv");

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
			.form(&create_plane(100.0, 100.0, -Vec3::Y, vec3(0.0, 20.0, 0.0)))
			.create();
		let wall_form = p
			.form(&create_plane(20.5, 5.0, Vec3::Z, vec3(15.0, 3.0, 0.0)))
			.create();

		let ground_shape = p.shape(ground_form, wall_render_shade).create();
		let wall_shape = p.shape(wall_form, wall_render_shade).create();
		let roof_shape = p.shape(roof_form, wall_render_shade).create();
		let grid_row_shape = p.shape(grid_row_form, wall_render_shade).create();
		let grid_col_shape = p.shape(grid_col_form, wall_render_shade).create();

		let vp_mat = p.bind_mat4();

		let canvas = p
			.layer()
			.with_shapes(vec![
				ground_shape,
				wall_shape,
				roof_shape,
				grid_row_shape,
				grid_col_shape,
			])
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_bindings(map! {
				0 => vp_mat.binding(),
			})
			.with_multisampling()
			.with_depth_test()
			.create();

		Self {
			cam,
			canvas,
			grid_col_tex,
			grid_row_tex,
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
		// p.paint_and_show(self.canvas)
		p.show(self.grid_row_tex)
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
