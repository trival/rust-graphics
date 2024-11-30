use std::time::Instant;

use geom::create_ball_geom;
use trivalibs::{
	hashmap,
	painter::{
		create_canvas_app,
		form::Form,
		painter::Mat3U,
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
	mvp: Uniform<Mat4>,
	norm: Uniform<Mat3U>,
	cam: PerspectiveCamera,
	ball_transform: Transform,
}

struct App {
	state: Option<InitializedState>,
	now: Instant,
}
impl Default for App {
	fn default() -> Self {
		Self {
			state: None,
			now: Instant::now(),
		}
	}
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
			uniform_layout: &[&uniform_layout, &uniform_layout],
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
		let mat = t.model_view_proj_mat(&cam);
		let norm = t.view_normal_mat(&cam);

		let mvp_uniform = painter.create_uniform_mat4(&uniform_layout, mat);
		let norm_uniform = painter.create_uniform_mat3(&uniform_layout, norm);

		self.state = Some(InitializedState {
			form: ball_form,
			shade,
			// tex_uniform: painter.get_texture_2d_uniform(&texture, &sampler),
			mvp: mvp_uniform,
			norm: norm_uniform,
			ball_transform: t,
			cam,
		});
	}

	fn render(&mut self, painter: &Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		let elapsed = self.now.elapsed().as_secs_f32();
		let state = self.state.as_mut().unwrap();

		state.ball_transform.rotate_y(elapsed * 5.);

		let mat = state.ball_transform.model_view_proj_mat(&state.cam);
		let norm = state.ball_transform.view_normal_mat(&state.cam);

		painter.update_uniform_mat4(&state.mvp, mat);
		painter.update_uniform_mat3(&state.norm, norm);

		painter.draw(
			&state.form,
			&state.shade,
			hashmap! {
				// 0 => &state.tex_uniform,
				0 => &state.mvp.binding,
				1 => &state.norm.binding,
			},
		)?;

		self.now = Instant::now();
		painter.redraw();

		Ok(())
	}

	fn user_event(&mut self, _event: (), _painter: &Painter) {}
	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
