use geom::create_ball_geom;
use std::num::NonZeroU64;
use trivalibs::{
	hashmap,
	painter::{
		create_canvas_app,
		painter::{Form, Shade, ShadeProps},
		CanvasApp, Painter,
	},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
		transform::Transform,
	},
	wgpu::{self, include_spirv, util::DeviceExt, VertexFormat::*},
	winit::event::{DeviceEvent, WindowEvent},
};

mod geom;

#[apply(gpu_data)]
pub struct Vertex {
	pub position: Vec3,
	pub color: Vec3,
	pub normal: Vec3,
}

struct InitializedState {
	form: Form,
	shade: Shade,
	// tex_uniform: wgpu::BindGroup,
	cam_uniforms: wgpu::BindGroup,
}

#[derive(Default)]
struct App {
	state: Option<InitializedState>,
}

impl CanvasApp<()> for App {
	fn init(&mut self, painter: &mut Painter) {
		// Initialize the app

		// let tex_bytes = include_bytes!("../texture.png");
		// let mut reader = png::Decoder::new(std::io::Cursor::new(tex_bytes))
		// 	.read_info()
		// 	.unwrap();
		// // Allocate the output buffer.
		// let mut buf = vec![0; reader.output_buffer_size()];
		// // Read the next frame. An APNG might contain multiple frames.
		// let info = reader.next_frame(&mut buf).unwrap();
		// // Grab the bytes of the image.
		// let tex_rgba = &buf[..info.buffer_size()];

		// let texture = painter.create_texture_2d(&Texture2DProps {
		// 	width: info.width,
		// 	height: info.height,
		// 	format: wgpu::TextureFormat::Rgba8UnormSrgb,
		// 	usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		// });

		// let sampler = painter.create_sampler(&SamplerProps {
		// 	address_mode_u: wgpu::AddressMode::ClampToEdge,
		// 	address_mode_v: wgpu::AddressMode::ClampToEdge,
		// 	mag_filter: wgpu::FilterMode::Linear,
		// 	min_filter: wgpu::FilterMode::Linear,
		// });

		// painter.fill_texture_2d(&texture, tex_rgba);

		let ball_buf = create_ball_geom();

		let ball_form = painter.create_form_with_size(
			(ball_buf.vertex_count * 3 * std::mem::size_of::<Vec3>() as u32) as u64,
		);
		painter.update_form_buffer(&ball_form, ball_buf);

		let size = painter.canvas_size();
		let cam = PerspectiveCamera::create(CamProps {
			aspect_ratio: Some(size.width as f32 / size.height as f32),
			fov: Some(0.6),
			..default()
		});

		let t = Transform::default();

		let uniforms = shader::Uniforms {
			mvp_mat: cam.view_proj_mat(),
			normal_mat: Mat4::from_mat3(t.view_normal_mat(&cam)),
		};

		let uniforms_buffer = painter
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Camera Buffer"),
				contents: bytemuck::cast_slice(&[uniforms]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			});

		let uniforms_bind_group_layout =
			painter
				.device
				.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
					entries: &[wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: (NonZeroU64::new(std::mem::size_of::<shader::Uniforms>() as u64)),
						},
						count: None,
					}],
					label: Some("uniforms_bind_group_layout"),
				});

		let uniforms_bind_group = painter
			.device
			.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: &uniforms_bind_group_layout,
				entries: &[wgpu::BindGroupEntry {
					binding: 0,
					resource: uniforms_buffer.as_entire_binding(),
				}],
				label: Some("camera_bind_group"),
			});

		let shade = painter.create_shade(ShadeProps {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![Float32x3, Float32x3, Float32x3],
			uniform_layout: &[&uniforms_bind_group_layout],
		});

		self.state = Some(InitializedState {
			form: ball_form,
			shade,
			// tex_uniform: painter.get_texture_2d_uniform(&texture, &sampler),
			cam_uniforms: uniforms_bind_group,
		});
	}

	fn render(&self, painter: &Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		let state = self.state.as_ref().unwrap();
		painter.draw(
			&state.form,
			&state.shade,
			hashmap! {
				// 0 => &state.tex_uniform,
				0 => &state.cam_uniforms
			},
		)
	}

	fn user_event(&mut self, _event: (), _painter: &Painter) {}
	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
