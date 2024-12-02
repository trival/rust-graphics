use geom::create_ball_geom;
use trivalibs::{
	hashmap,
	painter::{
		create_canvas_app,
		shade::ShadeProps,
		sketch::{Sketch, SketchProps},
		texture::{SamplerProps, Texture2DProps},
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

#[apply(gpu_data)]
pub struct Vertex {
	pub position: Vec3,
	pub color: Vec3,
	pub normal: Vec3,
}

struct RenderState {
	sketch: Sketch,
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
	fn init(&mut self, painter: &mut Painter) -> RenderState {
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

		let texture = painter.texture_2d_create(&Texture2DProps {
			width: info.width,
			height: info.height,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});

		let sampler = painter.create_sampler(&SamplerProps {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
		});

		painter.texture_2d_fill(texture, tex_rgba);

		let uniform_layout = painter.uniform_get_layout_buffered(wgpu::ShaderStages::VERTEX);
		let tex_layout = painter.texture_2d_get_uniform_layout(wgpu::ShaderStages::FRAGMENT);

		let shade = painter.shade_create(ShadeProps {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![Float32x3, Float32x2, Float32x3, Float32x3],
			uniform_layout: &[&uniform_layout, &uniform_layout, &tex_layout],
		});

		let form = painter.from_from_buffer(create_ball_geom(), default());

		let mvp = painter.uniform_create_mat4(&uniform_layout, Mat4::IDENTITY);
		let norm = painter.uniform_create_mat3(&uniform_layout, Mat3::IDENTITY);
		let tex = painter.uniform_create_tex(&tex_layout, texture, &sampler);

		let sketch = painter.sketch_create(
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

		RenderState { sketch, mvp, norm }
	}

	fn resize(&mut self, painter: &Painter) {
		let size = painter.canvas_size();

		self
			.cam
			.set_aspect_ratio(size.width as f32 / size.height as f32);
	}

	fn update(&mut self, painter: &mut Painter, render_state: &mut RenderState, tpf: f32) {
		self.ball_transform.rotate_y(tpf * 0.5);

		painter.uniform_update_mat4(
			&render_state.mvp,
			self.ball_transform.model_view_proj_mat(&self.cam),
		);
		painter.uniform_update_mat3(
			&render_state.norm,
			self.ball_transform.view_normal_mat(&self.cam),
		);
	}

	fn render(
		&self,
		painter: &Painter,
		state: &RenderState,
	) -> std::result::Result<(), wgpu::SurfaceError> {
		painter.request_redraw();
		painter.draw(&state.sketch)
	}

	fn user_event(&mut self, _event: (), _painter: &Painter) {}
	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
