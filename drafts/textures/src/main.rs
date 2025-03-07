use std::any::Any;

use trivalibs::{map, painter::prelude::*, prelude::*};
use utils::textures_f32;

mod utils;

fn no_op(layer: Layer) {
	let _ = layer.type_id();
}

struct App {
	time: f32,
	u_size: UniformBuffer<UVec2>,
	u_time: UniformBuffer<f32>,

	canvas_simplex_prefilled: Layer,
	canvas_simplex_shader: Layer,
	canvas_fbm_shader: Layer,

	canvas_bos_shaping_fns_1: Layer,
}

const NOISE_TEXTURE_WIDTH: u32 = 112;
const NOISE_TEXTURE_HEIGHT: u32 = 512;

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let (_tex_rand, tex_simplex) = textures_f32(p, NOISE_TEXTURE_WIDTH, NOISE_TEXTURE_HEIGHT, 0.6);

		let sampler = p
			.sampler()
			.with_filters(wgpu::FilterMode::Linear)
			.with_address_modes(wgpu::AddressMode::Repeat)
			.create();

		let u_size = p.uniform_uvec2();

		let u_time = p.uniform_f32();

		// simplex shader

		let s = p
			.shade_effect()
			.with_uniforms(&[UNIFORM_BUFFER_FRAG, UNIFORM_BUFFER_FRAG])
			.create();
		load_fragment_shader!(s, p, "../shader/simplex_shader_frag.spv");

		let e = p.effect(s).create();
		let canvas_simplex_shader = p
			.layer()
			.with_effect(e)
			.with_uniforms(map! {
				0 => u_size.uniform(),
				1 => u_time.uniform()
			})
			.create();

		// fbm shader

		let s = p
			.shade_effect()
			.with_uniforms(&[UNIFORM_BUFFER_FRAG, UNIFORM_BUFFER_FRAG])
			.create();
		load_fragment_shader!(s, p, "../shader/fbm_shader_frag.spv");

		let e = p.effect(s).create();
		let canvas_fbm_shader = p
			.layer()
			.with_effect(e)
			.with_uniforms(map! {
				0 => u_size.uniform(),
				1 => u_time.uniform()
			})
			.create();

		// simplex prefilled

		let s = p
			.shade_effect()
			.with_uniforms(&[
				UNIFORM_TEX2D_FRAG,
				UNIFORM_SAMPLER_FRAG,
				UNIFORM_BUFFER_FRAG,
			])
			.create();
		load_fragment_shader!(s, p, "../shader/simplex_prefilled_frag.spv");

		let e = p.effect(s).create();
		let canvas_simplex_prefilled = p
			.layer()
			.with_effect(e)
			.with_uniforms(map! {
				0 => tex_simplex.uniform(),
				1 => sampler.uniform(),
				2 => u_size.uniform()
			})
			.create();

		// bos shaping fns 1

		let s = p.shade_effect().create();
		load_fragment_shader!(s, p, "../shader/bos_shaping_fns_1.spv");

		let e = p.effect(s).create();
		let canvas_bos_shaping_fns_1 = p.layer().with_effect(e).create();

		// return App

		Self {
			time: 0.0,
			u_size,
			u_time,

			canvas_simplex_shader,
			canvas_simplex_prefilled,
			canvas_fbm_shader,

			canvas_bos_shaping_fns_1,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		no_op(self.canvas_simplex_prefilled);
		no_op(self.canvas_simplex_shader);
		no_op(self.canvas_fbm_shader);
		no_op(self.canvas_bos_shaping_fns_1);

		p.paint_and_show(self.canvas_fbm_shader)
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);
		p.request_next_frame();
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
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
