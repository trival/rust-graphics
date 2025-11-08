use trivalibs::{painter::prelude::*, prelude::*};

struct App {
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p.shade_effect().create();
		load_fragment_shader!(shade, p, "../shader/main.spv");

		let effect = p.effect(shade).create();

		let canvas = p.layer().with_effect(effect).create();

		Self { canvas }
	}

	fn resize(&mut self, p: &mut Painter, _width: u32, _height: u32) {
		p.request_next_frame();
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {
		// p.request_next_frame(); // request frame here instead of resize for constant animation loop
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint_and_show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			remember_window_dimensions: true,
			..default()
		})
		.start();
}
