use draft_polygons_shader::Vertex;
use glam::vec3;
use trival_painter::{create_app, Application, Painter};
use wgpu::{include_spirv, util::DeviceExt};
use winit::event::{DeviceEvent, WindowEvent};

struct InitializedState {
	pipeline: wgpu::RenderPipeline,
	buffer: wgpu::Buffer,
}

const VERTICES: &[Vertex] = &[
	Vertex {
		position: vec3(0.0, 0.5, 0.0),
		color: vec3(1.0, 0.0, 0.0),
	},
	Vertex {
		position: vec3(-0.5, -0.5, 0.0),
		color: vec3(0.0, 1.0, 0.0),
	},
	Vertex {
		position: vec3(0.5, -0.5, 0.0),
		color: vec3(0.0, 0.0, 1.0),
	},
];

#[derive(Default)]
struct App {
	state: Option<InitializedState>,
}

impl Application<()> for App {
	fn init(&mut self, painter: &Painter) {
		// Initialize the app

		let pipeline_layout = painter
			.device
			.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});

		// Load the shaders from disk
		let vert_shader = painter
			.device
			.create_shader_module(include_spirv!("../shader/vertex.spv"));
		let frag_shader = painter
			.device
			.create_shader_module(include_spirv!("../shader/fragment.spv"));

		let pipeline = painter
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &vert_shader,
					entry_point: None,
					buffers: &[wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
					}],
					compilation_options: Default::default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &frag_shader,
					entry_point: None,
					compilation_options: Default::default(),
					targets: &[Some(wgpu::ColorTargetState {
						format: painter.config.format, // for direct rendering into te surface
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					})],
				}),
				primitive: Default::default(),
				depth_stencil: None,
				multisample: Default::default(),
				multiview: None,
				cache: None,
			});

		let buffer = painter
			.device
			.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(VERTICES),
				usage: wgpu::BufferUsages::VERTEX,
			});

		self.state = Some(InitializedState { pipeline, buffer });
	}

	fn render(&self, painter: &Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		let state = self.state.as_ref().unwrap();
		let frame = painter.surface.get_current_texture()?;

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = painter
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			rpass.set_pipeline(&state.pipeline);
			rpass.set_vertex_buffer(0, state.buffer.slice(..));
			rpass.draw(0..3, 0..1);
		}

		painter.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	fn user_event(&mut self, _event: (), _painter: &Painter) {}
	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
}

pub fn main() {
	create_app(App::default()).start();
}
