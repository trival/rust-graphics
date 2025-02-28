use trivalibs::{map, painter::prelude::*, prelude::*};
use utils::tiled_noise_rgba_u8;

mod utils;

struct App {
	u_size: UniformBuffer<UVec2>,
	canvas_simplex_shader: Layer,
	canvas_simplex_prefilled: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let texture = p.texture_2d_create(Texture2DProps {
			width: 512,
			height: 256,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});

		texture.fill_2d(p, &tiled_noise_rgba_u8(512, 256, 0.1));

		let shade = p.shade_create_effect(ShadeEffectProps {
			uniforms: &[UNIFORM_BUFFER_FRAG],
			layers: &[],
		});
		load_fragment_shader!(shade, p, "../tex1_shader/main.spv");

		let u_size = p.uniform_uvec2();

		let canvas_simplex_shader = create_shader_canvas(p, shade, &u_size);
		let canvas_simplex_prefilled = create_shader_canvas(p, shade, &u_size);

		Self {
			u_size,
			canvas_simplex_shader,
			canvas_simplex_prefilled,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint(self.canvas_simplex_shader)?;
		p.paint(self.canvas_simplex_prefilled)?;
		p.show(self.canvas_simplex_shader)
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}
	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

fn create_shader_canvas(p: &mut Painter, shade: Shade, u_size: &UniformBuffer<UVec2>) -> Layer {
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
	canvas
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: true,
		})
		.start();
}
