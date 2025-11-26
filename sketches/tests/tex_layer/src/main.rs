use shared::static_effect_layer_u8;
use trivalibs::{painter::prelude::*, prelude::*};

struct App {
	canvas: Layer,
	time: f32,
	color: BindingBuffer<Vec3U>,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let color = p.bind_vec3();

		let (canvas, shade) = static_effect_layer_u8(p, 10, 10, map! { 1 => color.binding() });
		load_fragment_shader!(shade, p, "../shader/main.spv");

		Self {
			canvas,
			time: 0.0,
			color,
		}
	}

	fn resize(&mut self, _p: &mut Painter, _width: u32, _height: u32) {}

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		p.request_next_frame();
		if self.time % 1.0 < 0.05 {
			self.color.update_vec3(p, rand_vec3());
		}
		self.time += tpf;
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
