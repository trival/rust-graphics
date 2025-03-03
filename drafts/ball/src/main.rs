use geom::create_ball_geom;
use trivalibs::{
	map,
	math::transform::Transform,
	painter::prelude::*,
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
	},
};

mod geom;

struct App {
	cam: PerspectiveCamera,
	ball_transform: Transform,

	mvp: UniformBuffer<Mat4>,
	norm: UniformBuffer<Mat3U>,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let tex_bytes = include_bytes!("../texture.png");
		let mut reader = png::Decoder::new(std::io::Cursor::new(tex_bytes))
			.read_info()
			.unwrap();
		// Allocate the output buffer.
		let mut buf = vec![0; reader.output_buffer_size()];
		// Read the next frame. An APNG might contain multiple frames.
		let info = reader.next_frame(&mut buf).unwrap();
		// Grab the bytes of the image.
		let tex_rgba = &buf[..info.buffer_size()];

		let texture = p.texture_2d(info.width, info.height).create();

		texture.fill_2d(p, tex_rgba);

		let shade = p
			.shade(&[Float32x3, Float32x2, Float32x3, Float32x3])
			.with_uniforms(&[
				UNIFORM_BUFFER_VERT,
				UNIFORM_BUFFER_VERT,
				UNIFORM_TEX2D_FRAG,
				UNIFORM_SAMPLER_FRAG,
			])
			.create();
		load_vertex_shader!(shade, p, "../shader/vertex.spv");
		load_fragment_shader!(shade, p, "../shader/fragment.spv");

		let form = p.form(&create_ball_geom()).create();

		let mvp = p.uniform_mat4();
		let norm = p.uniform_mat3();

		let s = p.sampler_linear().uniform();

		let shape = p
			.shape(form, shade)
			.with_uniforms(map! {
				0 => mvp.uniform(),
				1 => norm.uniform(),
				2 => texture.uniform(),
				3 => s
			})
			.create();

		let canvas = p
			.layer()
			.with_shapes(vec![shape])
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_multisampling()
			.create();

		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				..default()
			}),
			ball_transform: Transform::from_translation(vec3(0.0, 0.0, -20.0)),
			canvas,
			mvp,
			norm,
		}
	}

	fn resize(&mut self, _p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.ball_transform.rotate_y(tpf * 0.5);

		self
			.mvp
			.update(p, self.ball_transform.model_view_proj_mat(&self.cam));

		self
			.norm
			.update_mat3(p, self.ball_transform.view_normal_mat(&self.cam));

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		p.paint_and_show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
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
