use trivalibs::{
	map,
	painter::{effect::Effect, prelude::*},
	prelude::*,
};
use utils::{rand_rgba_u8, tiled_noise_rgba_u8};

mod utils;

struct App {
	u_size: UniformBuffer<UVec2>,
	canvas_simplex_shader: Layer,
	canvas_simplex_prefilled: Layer,
}

const NOISE_TEXTURE_WIDTH: u32 = 512;
const NOISE_TEXTURE_HEIGHT: u32 = 512;

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let texture_simplex = p.texture_2d_create(Texture2DProps {
			width: NOISE_TEXTURE_WIDTH,
			height: NOISE_TEXTURE_HEIGHT,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});

		texture_simplex.fill_2d(
			p,
			&tiled_noise_rgba_u8(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT, 0.3),
		);

		let texture_random = p.texture_2d_create(Texture2DProps {
			width: NOISE_TEXTURE_WIDTH,
			height: NOISE_TEXTURE_HEIGHT,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});

		texture_random.fill_2d(p, &rand_rgba_u8(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT));

		let sampler = p.sampler_create(SamplerProps {
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			..default()
		});

		let u_size = p.uniform_uvec2();

		let shade_simplex_shader = p.shade_create_effect(ShadeEffectProps {
			uniforms: &[UNIFORM_BUFFER_FRAG],
			layers: &[],
		});
		load_fragment_shader!(shade_simplex_shader, p, "../shader/simplex_shader_frag.spv");

		let effect_simplex_shader = p.effect_create(
			shade_simplex_shader,
			EffectProps {
				uniforms: map! {
					0 => u_size.uniform()
				},
				..default()
			},
		);

		let shade_simplex_prefilled = p.shade_create_effect(ShadeEffectProps {
			uniforms: &[
				UNIFORM_TEX2D_FRAG,
				UNIFORM_SAMPLER_FRAG,
				UNIFORM_BUFFER_FRAG,
			],
			layers: &[],
		});
		load_fragment_shader!(
			shade_simplex_prefilled,
			p,
			"../shader/simplex_prefilled_frag.spv"
		);

		let effect_simplex_prefilled = p.effect_create(
			shade_simplex_prefilled,
			EffectProps {
				uniforms: map! {
					0 => texture_simplex.uniform(),
					1 => sampler.uniform(),
					2 => u_size.uniform()
				},
				..default()
			},
		);

		let canvas_simplex_shader = create_shader_canvas(p, effect_simplex_shader);
		let canvas_simplex_prefilled = create_shader_canvas(p, effect_simplex_prefilled);

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
		p.show(self.canvas_simplex_prefilled)
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}
	fn event(&mut self, e: Event<()>, p: &mut Painter) {
		match e {
			Event::ShaderReloadEvent => {
				p.request_next_frame();
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
			keep_window_dimensions: true,
		})
		.start();
}

fn create_shader_canvas(p: &mut Painter, effect: Effect) -> Layer {
	let canvas = p.layer_create(LayerProps {
		effects: vec![effect],
		..default()
	});
	canvas
}
