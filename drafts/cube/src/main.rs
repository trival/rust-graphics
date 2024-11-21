use glam::{vec2, vec3, Vec2, Vec3};
use trival_painter::painter::{Form, FormProps, Shade, ShadeProps, Texture, Texture2DProps};
use trival_painter::{create_app, Application, Painter};
use trival_painter::{hashmap, macros::*};
use wgpu::{include_spirv, VertexFormat::*};
use winit::event::{DeviceEvent, WindowEvent};

#[apply(gpu_data)]
pub struct Vertex {
	pub position: Vec3,
	pub color: Vec3,
	pub uv: Vec2,
}

struct InitializedState {
	form: Form,
	shade: Shade,
	texture: Texture,
}

const VERTICES: &[Vertex] = &[
	Vertex {
		position: vec3(0.0, 0.5, 0.0),
		color: vec3(1.0, 0.0, 0.0),
		uv: vec2(0.5, 1.0),
	},
	Vertex {
		position: vec3(-0.5, -0.5, 0.0),
		color: vec3(0.0, 1.0, 0.0),
		uv: vec2(0.0, 0.0),
	},
	Vertex {
		position: vec3(0.5, -0.5, 0.0),
		color: vec3(0.0, 0.0, 1.0),
		uv: vec2(1.0, 0.0),
	},
];

#[derive(Default)]
struct App {
	state: Option<InitializedState>,
}

impl Application<()> for App {
	fn init(&mut self, painter: &mut Painter) {
		// Initialize the app

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

		let texture = painter.create_texture_2d(Texture2DProps {
			width: info.width,
			height: info.height,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			data: tex_rgba,
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
		});

		let shade = painter.create_shade(ShadeProps {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![Float32x3, Float32x3, Float32x2],
			uniform_layout: &[&painter.get_texture_2d_uniform_layout()],
		});

		let form = painter.create_form(FormProps {
			vertex_buffer: VERTICES,
			index_buffer: None,
		});

		self.state = Some(InitializedState {
			form,
			shade,
			texture,
		});
	}

	fn render(&self, painter: &Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		let state = self.state.as_ref().unwrap();
		painter.draw(
			&state.form,
			&state.shade,
			hashmap! { 0 => painter.get_texture_uniform(&state.texture)  },
		)
	}

	fn user_event(&mut self, _event: (), _painter: &Painter) {}
	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
}

pub fn main() {
	create_app(App::default()).start();
}
