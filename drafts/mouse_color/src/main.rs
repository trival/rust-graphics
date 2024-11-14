use trival_renderer::{create_app, Application, Renderer};
use winit::event::{DeviceEvent, WindowEvent};

struct InitializedState {
	color: wgpu::Color,
}

#[derive(Default)]
struct App {
	state: Option<InitializedState>,
}

struct UserEvent(wgpu::Color);

impl Application<UserEvent> for App {
	fn init(&mut self, _ctx: &Renderer) {
		// Initialize the app

		self.state = Some(InitializedState {
			color: wgpu::Color {
				r: 0.1,
				g: 0.2,
				b: 0.3,
				a: 1.0,
			},
		});
	}

	fn render(&self, renderer: &Renderer) -> std::result::Result<(), wgpu::SurfaceError> {
		let state = self.state.as_ref().unwrap();
		let frame = renderer.surface.get_current_texture()?;

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = renderer
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
		}

		renderer.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	fn user_event(&mut self, event: UserEvent, renderer: &Renderer) {
		let state = self.state.as_mut().unwrap();
		state.color = event.0;
		renderer.request_redraw();
	}

	fn window_event(&mut self, event: WindowEvent, renderer: &Renderer) {
		match event {
			WindowEvent::CursorMoved {
				device_id: _,
				position,
			} => {
				let size = renderer.window.inner_size();
				let state = self.state.as_mut().unwrap();
				state.color = wgpu::Color {
					r: position.x / size.width as f64,
					g: position.y / size.height as f64,
					b: 0.3,
					a: 1.0,
				};
				renderer.request_redraw();
			}
			_ => {}
		}
	}
	fn device_event(&mut self, _event: DeviceEvent, _renderer: &Renderer) {}
}

pub fn main() {
	create_app(App::default()).start();
}
