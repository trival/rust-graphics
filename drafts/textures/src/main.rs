use trivalibs::{
	map,
	painter::{
		prelude::*,
		texture::Texture,
		winit::event::{ElementState, MouseButton, WindowEvent},
	},
	prelude::*,
};
use utils::textures_f32;

mod utils;

#[derive(Copy, Clone)]
struct Canvas {
	layer: Layer,
	animated: bool,
}

struct App {
	time: f32,
	u_size: UniformBuffer<UVec2>,
	u_time: UniformBuffer<f32>,

	canvases: Vec<Canvas>,
	current_canvas: usize,
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

		let texture_shade_canvas = |p: &mut Painter, tex: Texture, animated: bool| {
			let s = p
				.shade_effect()
				.with_uniforms(&[
					UNIFORM_TEX2D_FRAG,
					UNIFORM_SAMPLER_FRAG,
					UNIFORM_BUFFER_FRAG,
				])
				.create();

			let e = p.effect(s).create();
			let layer = p
				.layer()
				.with_effect(e)
				.with_uniforms(map! {
					0 => tex.uniform(),
					1 => sampler.uniform(),
					2 => u_size.uniform()
				})
				.create();

			(s, Canvas { layer, animated })
		};

		let shade_canvas = |p: &mut Painter, animated: bool| {
			let s = p
				.shade_effect()
				.with_uniforms(&[UNIFORM_BUFFER_FRAG, UNIFORM_BUFFER_FRAG])
				.create();

			let e = p.effect(s).create();
			let layer = p
				.layer()
				.with_effect(e)
				.with_uniforms(map! {
					0 => u_size.uniform(),
					1 => u_time.uniform()
				})
				.create();

			(s, Canvas { layer, animated })
		};

		let (s, canvas_simplex_shader) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/simplex_shader.spv");

		let (s, canvas_fbm_shader) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/fbm_shader.spv");

		let (s, canvas_simplex_prefilled) = texture_shade_canvas(p, tex_simplex, false);
		load_fragment_shader!(s, p, "../shader/simplex_prefilled.spv");

		let (s, canvas_bos_shaping_fns) = shade_canvas(p, false);
		load_fragment_shader!(s, p, "../shader/bos_shaping_fns.spv");

		let (s, canvas_bos_colors) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/bos_colors.spv");

		let (s, canvas_bos_shapes_rect) = shade_canvas(p, false);
		load_fragment_shader!(s, p, "../shader/bos_shapes_rect.spv");

		let (s, canvas_bos_shapes_circle) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/bos_shapes_circle.spv");

		let (s, canvas_bos_shapes_circles) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/bos_shapes_circles.spv");

		let (s, canvas_hash_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/hash_test.spv");

		let (s, canvas_tiles) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/tiled_plates.spv");

		let (s, canvas_noisy_lines_1) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/noisy_lines_1.spv");

		let (s, canvas_tiled_lines) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/tiled_lines.spv");

		// return App

		Self {
			time: 0.0,
			u_size,
			u_time,

			canvases: vec![
				canvas_tiles,
				canvas_tiled_lines,
				canvas_noisy_lines_1,
				canvas_hash_test,
				canvas_bos_shapes_circles,
				canvas_bos_shapes_circle,
				canvas_bos_shapes_rect,
				canvas_bos_colors,
				canvas_bos_shaping_fns,
				canvas_fbm_shader,
				canvas_simplex_prefilled,
				canvas_simplex_shader,
			],
			current_canvas: 0,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		let c = &self.canvases[self.current_canvas];
		if c.animated {
			p.request_next_frame();
		}
		p.paint_and_show(c.layer)
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);
	}

	fn event(&mut self, e: Event<()>, p: &mut Painter) {
		match e {
			Event::ShaderReloadEvent => {
				p.request_next_frame();
			}
			Event::WindowEvent(WindowEvent::MouseInput { state, button, .. }) => {
				if state == ElementState::Released {
					p.request_next_frame();
					match button {
						MouseButton::Left => {
							self.current_canvas = (self.current_canvas + 1) % self.canvases.len();
						}
						_ => {
							self.current_canvas =
								(self.current_canvas + self.canvases.len() - 1) % self.canvases.len();
						}
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
			keep_window_dimensions: true,
			features: Some(wgpu::Features::FLOAT32_FILTERABLE),
		})
		.start();
}
