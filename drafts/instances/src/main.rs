use trivalibs::{
	painter::{create_canvas_app, CanvasApp, Painter},
	prelude::*,
	wgpu,
	winit::event::{DeviceEvent, WindowEvent},
};

const VERTICES: &[Vec3] = &[vec3(0.0, 5.0, 0.0), vec3(-2.5, 0., 0.0), vec3(2.5, 0., 0.0)];

#[derive(Default)]
struct App {}

struct RenderState {}

impl CanvasApp<RenderState, ()> for App {
	fn init(&mut self, painter: &mut Painter) -> RenderState {
		RenderState {}
	}

	fn resize(&mut self, painter: &Painter) {}

	fn update(&mut self, painter: &mut Painter, render_state: &mut RenderState, tpf: f32) {}

	fn render(
		&self,
		painter: &Painter,
		render_state: &RenderState,
	) -> Result<(), wgpu::SurfaceError> {
		todo!()
	}

	fn window_event(&mut self, event: WindowEvent, painter: &Painter) {}
	fn device_event(&mut self, event: DeviceEvent, painter: &Painter) {}
	fn user_event(&mut self, event: (), painter: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
