use geom::create_ball_geom;
use trivalibs::{
	hashmap,
	painter::{
		create_canvas_app,
		form::Form,
		shade::{Shade, ShadeProps},
		uniform::Uniform,
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

struct InitializedState {
	form: Form,
	shade: Shade,
	// tex_uniform: wgpu::BindGroup,
	mvp_uniform: Uniform<Mat4>,
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

		let uniform_layout = painter.get_uniform_layout_buffered(wgpu::ShaderStages::VERTEX);

		let shade = painter.create_shade(ShadeProps {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![Float32x3, Float32x3, Float32x3],
			uniform_layout: &[&uniform_layout],
		});

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

		let t = Transform::from_translation(vec3(0.0, 0.0, -20.0));

		// let uniforms = shader::Uniforms {
		// 	mvp_mat: cam.view_proj_mat(),
		// 	normal_mat: Mat4::from_mat3(t.view_normal_mat(&cam)),
		// };

		let mat = t.model_view_proj_mat(&cam);

		let uniform = painter.create_uniform_buffered(&uniform_layout, mat);

		self.state = Some(InitializedState {
			form: ball_form,
			shade,
			// tex_uniform: painter.get_texture_2d_uniform(&texture, &sampler),
			mvp_uniform: uniform,
		});
	}

	fn render(&mut self, painter: &Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		let state = self.state.as_ref().unwrap();
		painter.draw(
			&state.form,
			&state.shade,
			hashmap! {
				// 0 => &state.tex_uniform,
				0 => &state.mvp_uniform
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
