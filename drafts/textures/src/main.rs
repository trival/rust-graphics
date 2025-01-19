use trivalibs::{map, painter::prelude::*, prelude::*};

struct App {
	u_size: UniformBuffer<UVec2>,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p.shade_create_effect(ShadeEffectProps {
			uniforms: &[UNIFORM_BUFFER_FRAG],
			layers: &[],
		});
		load_fragment_shader!(shade, p, "../tex1_shader/main.spv");

		let u_size = p.uniform_uvec2();

		let effect = p.effect_create(
			shade,
			EffectProps {
				uniforms: map! {
					0 => u_size.uniform()
				},
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			effects: vec![effect],
			..default()
		});

		Self { u_size, canvas }
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint_and_show(self.canvas)
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}
	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: true,
		})
		.start();
}
