use trival_renderer::{Application, ApplicationRunner, Renderer};
use wgpu::include_spirv;

struct InitializedState {
	pipeline: wgpu::RenderPipeline,
}

#[derive(Default)]
struct App {
	state: Option<InitializedState>,
}

struct UserEvent;

impl Application<UserEvent> for App {
	fn init(&mut self, ctx: &Renderer) {
		// Initialize the app

		let pipeline_layout = ctx
			.device
			.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});

		let swapchain_capabilities = ctx.surface.get_capabilities(&ctx.adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		// Load the shaders from disk
		let vert_shader = ctx
			.device
			.create_shader_module(include_spirv!("../shader/vertex.spv"));
		let frag_shader = ctx
			.device
			.create_shader_module(include_spirv!("../shader/fragment.spv"));

		let pipeline = ctx
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &vert_shader,
					entry_point: None,
					buffers: &[],
					compilation_options: Default::default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &frag_shader,
					entry_point: None,
					compilation_options: Default::default(),
					targets: &[Some(swapchain_format.into())],
				}),
				primitive: wgpu::PrimitiveState::default(),
				depth_stencil: None,
				multisample: wgpu::MultisampleState::default(),
				multiview: None,
				cache: None,
			});

		self.state = Some(InitializedState { pipeline });
	}

	fn render(&self, renderer: &Renderer) {
		let state = self.state.as_ref().unwrap();
		let frame = renderer
			.surface
			.get_current_texture()
			.expect("Failed to acquire next swap chain texture");

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = renderer
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			rpass.set_pipeline(&state.pipeline);
			rpass.draw(0..3, 0..1);
		}

		renderer.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn user_event(&mut self, _event: UserEvent) {}
	fn window_event(&mut self, _event: winit::event::WindowEvent) {}
	fn device_event(&mut self, _event: winit::event::DeviceEvent) {}
}

pub fn main() {
	ApplicationRunner::start(App::default());
}
