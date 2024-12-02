use trivalibs::{
	painter::{create_canvas_app, CanvasApp, Painter},
	wgpu::{self, include_spirv},
	winit::event::{DeviceEvent, WindowEvent},
};

struct InitializedState {
	pipeline: wgpu::RenderPipeline,
	color: wgpu::Color,
}

#[derive(Default)]
struct App {
	state: Option<InitializedState>,
}

struct UserEvent(wgpu::Color);

impl CanvasApp<UserEvent> for App {
	fn init(&mut self, painter: &mut Painter) {
		// Initialize the app

		let pipeline_layout = painter
			.device
			.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});

		// let capabilities = painter.surface.get_capabilities(&painter.adapter);
		// let format = capabilities.formats[0];

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
					buffers: &[],
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
				primitive: wgpu::PrimitiveState::default(),
				depth_stencil: None,
				multisample: wgpu::MultisampleState::default(),
				multiview: None,
				cache: None,
			});

		self.state = Some(InitializedState {
			pipeline,
			color: wgpu::Color::BLUE,
		});
	}

	fn render(
		&mut self,
		painter: &Painter,
		_tpf: f32,
	) -> std::result::Result<(), wgpu::SurfaceError> {
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
						load: wgpu::LoadOp::Clear(state.color),
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

		painter.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	fn user_event(&mut self, event: UserEvent, painter: &Painter) {
		let state = self.state.as_mut().unwrap();
		state.color = event.0;
		painter.request_redraw();
	}

	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
}

pub fn main() {
	let app = create_canvas_app(App::default());
	let handle = app.get_handle();

	std::thread::spawn(move || loop {
		std::thread::sleep(std::time::Duration::from_secs(2));
		let _ = handle.send_event(UserEvent(wgpu::Color::RED));
		std::thread::sleep(std::time::Duration::from_secs(2));
		let _ = handle.send_event(UserEvent(wgpu::Color::GREEN));
		std::thread::sleep(std::time::Duration::from_secs(2));
		let _ = handle.send_event(UserEvent(wgpu::Color::BLUE));
	});

	app.start();
}
