use trivalibs::{map, painter::prelude::*, prelude::*};

struct App {
	time: f32,
	u_size: BindingBuffer<UVec2>,
	u_time: BindingBuffer<f32>,

	layers: Vec<Layer>,
	current_layer: usize,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let u_size = p.bind_uvec2();
		let u_time = p.bind_f32();

		let shade_canvas = |p: &mut Painter| {
			let s = p
				.shade_effect()
				.with_bindings(&[BINDING_BUFFER_FRAG, BINDING_BUFFER_FRAG])
				.create();

			let layer = p
				.single_effect_layer(s)
				.with_bindings(map! {
					0 => u_size.binding(),
					1 => u_time.binding()
				})
				.create();

			(s, layer)
		};

		let (s, canvas_noisy_lines_2) = shade_canvas(p);
		load_fragment_shader!(s, p, "../shader/out/lines_1.spv");

		// return App

		Self {
			time: 0.0,
			u_size,
			u_time,

			layers: vec![canvas_noisy_lines_2],
			current_layer: 0,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));
	}

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);

		let layer = self.layers[self.current_layer];
		p.paint_and_show(layer);

		p.request_next_frame();
	}

	fn event(&mut self, e: Event<()>, p: &mut Painter) {
		match e {
			Event::PointerUp { button, .. } => {
				p.request_next_frame();
				match button {
					PointerButton::Primary => {
						self.current_layer = (self.current_layer + 1) % self.layers.len();
					}
					_ => {
						self.current_layer = (self.current_layer + self.layers.len() - 1) % self.layers.len();
					}
				}
			}
			_ => {}
		}
	}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: true,
			remember_window_dimensions: true,
			features: Some(wgpu::Features::FLOAT32_FILTERABLE),
		})
		.start();
}
