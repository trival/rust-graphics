use glam::{vec2, vec3, Vec2, Vec3};
use trival_painter::painter::{Form, FormDescriptor, Shade, ShadeDescriptor};
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
	diffuse_bind_group: wgpu::BindGroup,
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
	fn init(&mut self, painter: &Painter) {
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
		let dimensions = (info.width, info.height);

		let texture_size = wgpu::Extent3d {
			width: dimensions.0,
			height: dimensions.1,
			depth_or_array_layers: 1,
		};

		let diffuse_texture = painter.device.create_texture(&wgpu::TextureDescriptor {
			// All textures are stored as 3D, we represent our 2D texture
			// by setting depth to 1.
			size: texture_size,
			mip_level_count: 1, // We'll talk about this a little later
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			// Most images are stored using sRGB, so we need to reflect that here.
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			// TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
			// COPY_DST means that we want to copy data to this texture
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			label: Some("diffuse_texture"),
			// This is the same as with the SurfaceConfig. It
			// specifies what texture formats can be used to
			// create TextureViews for this texture. The base
			// texture format (Rgba8UnormSrgb in this case) is
			// always supported. Note that using a different
			// texture format is not supported on the WebGL2
			// backend.
			view_formats: &[],
		});

		painter.queue.write_texture(
			// Tells wgpu where to copy the pixel data
			wgpu::ImageCopyTexture {
				texture: &diffuse_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			// The actual pixel data
			&tex_rgba,
			// The layout of the texture
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * dimensions.0),
				rows_per_image: Some(dimensions.1),
			},
			texture_size,
		);

		// We don't need to configure the texture view much, so let's
		// let wgpu define it.
		let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
		let diffuse_sampler = painter.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let texture_bind_group_layout = painter.create_uniform_layout_sampled_texture_2d();

		let diffuse_bind_group = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &texture_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
					},
				],
				label: Some("diffuse_bind_group"),
			});

		let shade = painter.create_shade(ShadeDescriptor {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![Float32x3, Float32x3, Float32x2],
			uniform_layout: &[&texture_bind_group_layout],
		});

		let form = painter.create_form(FormDescriptor {
			vertex_buffer: VERTICES,
			index_buffer: None,
		});

		self.state = Some(InitializedState {
			form,
			shade,
			diffuse_bind_group,
		});
	}

	fn render(&self, painter: &Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		let state = self.state.as_ref().unwrap();
		painter.draw(
			&state.form,
			&state.shade,
			hashmap! { 0 => &state.diffuse_bind_group },
		)
	}

	fn user_event(&mut self, _event: (), _painter: &Painter) {}
	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
}

pub fn main() {
	create_app(App::default()).start();
}
