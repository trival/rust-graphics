use std::any::Any;

use trivalibs::{
	map,
	painter::{prelude::*, texture::Texture},
	prelude::*,
};
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

	canvas_bos_shaping_fns: Layer,
	canvas_bos_shapes_rect: Layer,
}

const NOISE_TEXTURE_WIDTH: u32 = 256;
const NOISE_TEXTURE_HEIGHT: u32 = 256;

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

		let texture_shade_canvas = |p: &mut Painter, tex: Texture| {
			let s = p
				.shade_effect()
				.with_uniforms(&[
					UNIFORM_TEX2D_FRAG,
					UNIFORM_SAMPLER_FRAG,
					UNIFORM_BUFFER_FRAG,
				])
				.create();

			let e = p.effect(s).create();
			let c = p
				.layer()
				.with_effect(e)
				.with_uniforms(map! {
					0 => tex.uniform(),
					1 => sampler.uniform(),
					2 => u_size.uniform()
				})
				.create();

			(s, c)
		};

		let shade_canvas = |p: &mut Painter| {
			let s = p
				.shade_effect()
				.with_uniforms(&[UNIFORM_BUFFER_FRAG, UNIFORM_BUFFER_FRAG])
				.create();

			let e = p.effect(s).create();
			let c = p
				.layer()
				.with_effect(e)
				.with_uniforms(map! {
					0 => u_size.uniform(),
					1 => u_time.uniform()
				})
				.create();

			(s, c)
		};

		// simplex shader

		let (s, canvas_simplex_shader) = shade_canvas(p);
		load_fragment_shader!(s, p, "../shader/simplex_shader.spv");

		// fbm shader

		let (s, canvas_fbm_shader) = shade_canvas(p);
		load_fragment_shader!(s, p, "../shader/fbm_shader.spv");

		// simplex prefilled

		let (s, canvas_simplex_prefilled) = texture_shade_canvas(p, tex_simplex);
		load_fragment_shader!(s, p, "../shader/simplex_prefilled.spv");

		// bos shaping fns 1

		let (s, canvas_bos_shaping_fns) = shade_canvas(p);
		load_fragment_shader!(s, p, "../shader/bos_shaping_fns.spv");

		// bos shapes rect

		let (s, canvas_bos_shapes_rect) = shade_canvas(p);
		load_fragment_shader!(s, p, "../shader/bos_shapes_rect.spv");

		// return App

		Self {
			time: 0.0,
			u_size,
			u_time,

			canvas_simplex_shader,
			canvas_simplex_prefilled,
			canvas_fbm_shader,

			canvas_bos_shaping_fns,
			canvas_bos_shapes_rect,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));

		no_op(self.canvas_simplex_prefilled);
		no_op(self.canvas_simplex_shader);
		no_op(self.canvas_fbm_shader);
		no_op(self.canvas_bos_shaping_fns);
		no_op(self.canvas_bos_shapes_rect);
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint_and_show(self.canvas_bos_shapes_rect)
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
