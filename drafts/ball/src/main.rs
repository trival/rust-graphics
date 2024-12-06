use geom::create_ball_geom;
use trivalibs::{
	hashmap,
	painter::{
		create_canvas_app,
		layer::{Layer, LayerProps},
		shade::ShadeProps,
		sketch::SketchProps,
		texture::Texture2DProps,
		uniform::{Mat3U, UniformBuffer},
		CanvasApp, Painter,
	},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
		transform::Transform,
	},
	wgpu::{self, include_spirv, VertexFormat::*},
	winit::event::{DeviceEvent, WindowEvent},
};

mod geom;

struct RenderState {
	canvas: Layer,
	mvp: UniformBuffer<Mat4>,
	norm: UniformBuffer<Mat3U>,
}

struct App {
	cam: PerspectiveCamera,
	ball_transform: Transform,
}

impl Default for App {
	fn default() -> Self {
		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				..default()
			}),
			ball_transform: Transform::from_translation(vec3(0.0, 0.0, -20.0)),
		}
	}
}

impl CanvasApp<RenderState, ()> for App {
	fn init(&mut self, p: &mut Painter) -> RenderState {
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

		let texture = p.texture_2d_create(&Texture2DProps {
			width: info.width,
			height: info.height,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});

		p.texture_2d_fill(texture, tex_rgba);

		let uniform_layout = p.uniform_get_layout_buffered(wgpu::ShaderStages::VERTEX);
		let tex_layout = p.uniform_get_layout_tex_2d(wgpu::ShaderStages::FRAGMENT);

		let shade = p.shade_create(ShadeProps {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![Float32x3, Float32x2, Float32x3, Float32x3],
			uniform_layout: &[&uniform_layout, &uniform_layout, &tex_layout],
		});

		let form = p.form_from_buffer(create_ball_geom(), default());

		let sampler = p.sampler_create(&default());
		let tex = p.uniform_create_tex(&tex_layout, texture, &sampler);

		let mvp = p.uniform_create_mat4(&uniform_layout, Mat4::IDENTITY);
		let norm = p.uniform_create_mat3(&uniform_layout, Mat3::IDENTITY);

		let sketch = p.sketch_create(
			form,
			shade,
			&SketchProps {
				uniforms: hashmap! {
					0 => mvp.uniform,
					1 => norm.uniform,
					2 => tex.uniform,
				},
				..default()
			},
		);

		let canvas = p.layer_create(&LayerProps {
			clear_color: Some(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			}),
			sketches: vec![sketch],
			..default()
		});

		RenderState { canvas, mvp, norm }
	}

	fn resize(&mut self, p: &mut Painter, rs: &mut RenderState) {
		let size = p.canvas_size();

		self
			.cam
			.set_aspect_ratio(size.width as f32 / size.height as f32);

		// TODO: this needs to happen automatically!
		rs.canvas.on_window_resize(p);
	}

	fn update(&mut self, p: &mut Painter, rs: &mut RenderState, tpf: f32) {
		self.ball_transform.rotate_y(tpf * 0.5);

		rs.mvp
			.update(p, self.ball_transform.model_view_proj_mat(&self.cam));

		rs.norm
			.update_mat3(p, self.ball_transform.view_normal_mat(&self.cam));
	}

	fn render(
		&self,
		p: &mut Painter,
		state: &RenderState,
	) -> std::result::Result<(), wgpu::SurfaceError> {
		p.paint(&state.canvas)?;
		p.show(&state.canvas)?;

		p.request_redraw();

		Ok(())
	}

	fn user_event(&mut self, _e: (), _p: &Painter) {}
	fn window_event(&mut self, _e: WindowEvent, _p: &Painter) {}
	fn device_event(&mut self, _e: DeviceEvent, _p: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
