use trivalibs::{map, painter::prelude::*, prelude::*};
use utils::{rand_rgba_f32, rand_rgba_u8, tiled_noise_rgba_f32, tiled_noise_rgba_u8};

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
		let texture_simplex = p
			.texture_2d(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT)
			.create();

		texture_simplex.fill_2d(
			p,
			&tiled_noise_rgba_u8(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT, 0.3),
		);

		let texture_random = p
			.texture_2d(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT)
			.create();

		texture_random.fill_2d(p, &rand_rgba_u8(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT));

		let texture_simplex_f32 = p
			.texture_2d(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT)
			.with_format(wgpu::TextureFormat::Rgba32Float)
			.create();

		texture_simplex_f32.fill_2d(
			p,
			bytemuck::cast_slice(&tiled_noise_rgba_f32(
				NOISE_TEXTURE_WIDTH,
				NOISE_TEXTURE_HEIGHT,
				0.3,
			)),
		);

		let texture_random_f32 = p
			.texture_2d(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT)
			.with_format(wgpu::TextureFormat::Rgba32Float)
			.create();

		texture_random_f32.fill_2d(
			p,
			bytemuck::cast_slice(&rand_rgba_f32(NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT)),
		);

		let sampler = p
			.sampler()
			.with_filters(wgpu::FilterMode::Linear)
			.with_address_modes(wgpu::AddressMode::Repeat)
			.create();

		let u_size = p.uniform_uvec2();

		let shade_simplex_shader = p
			.shade_effect()
			.with_uniforms(&[UNIFORM_BUFFER_FRAG])
			.create();
		load_fragment_shader!(shade_simplex_shader, p, "../shader/simplex_shader_frag.spv");

		let effect_simplex_shader = p
			.effect(shade_simplex_shader)
			.with_uniforms(map! {
				0 => u_size.uniform()
			})
			.create();

		let shade_simplex_prefilled = p
			.shade_effect()
			.with_uniforms(&[
				UNIFORM_TEX2D_FRAG,
				UNIFORM_SAMPLER_FRAG,
				UNIFORM_BUFFER_FRAG,
			])
			.create();
		load_fragment_shader!(
			shade_simplex_prefilled,
			p,
			"../shader/simplex_prefilled_frag.spv"
		);

		let effect_simplex_prefilled = p
			.effect(shade_simplex_prefilled)
			.with_uniforms(map! {
				0 => texture_simplex_f32.uniform(),
				1 => sampler.uniform(),
				2 => u_size.uniform()
			})
			.create();

		let canvas_simplex_shader = p.layer().with_effect(effect_simplex_shader).create();
		let canvas_simplex_prefilled = p.layer().with_effect(effect_simplex_prefilled).create();

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
			features: Some(wgpu::Features::FLOAT32_FILTERABLE),
		})
		.start();
}
