use trivalibs::{
	painter::{create_canvas_app, CanvasApp, Painter},
	wgpu,
	winit::event::{DeviceEvent, WindowEvent},
};

struct App {
	color: wgpu::Color,
}

impl App {
	fn new() -> Self {
		Self {
			color: wgpu::Color {
				r: 0.3,
				g: 0.3,
				b: 0.3,
				a: 1.0,
			},
		}
	}
}

impl CanvasApp<()> for App {
	fn render(&self, painter: &Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		let frame = painter.surface.get_current_texture()?;

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = painter
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(self.color),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
		}

		painter.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	fn window_event(&mut self, event: WindowEvent, painter: &Painter) {
		match event {
			WindowEvent::CursorMoved {
				device_id: _,
				position,
			} => {
				let size = painter.canvas_size();
				self.color = wgpu::Color {
					r: position.x / size.width as f64,
					g: position.y / size.height as f64,
					b: 0.3,
					a: 1.0,
				};
				painter.redraw();
			}
			_ => {}
		}
	}

	fn init(&mut self, _painter: &mut Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
	fn user_event(&mut self, _event: (), _painter: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::new()).start();
}
