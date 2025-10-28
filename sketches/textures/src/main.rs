use trivalibs::{
	map,
	painter::{
		prelude::*,
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
	u_size: BindingBuffer<UVec2>,
	u_time: BindingBuffer<f32>,

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

		let u_size = p.bind_uvec2();

		let u_time = p.bind_f32();

		let texture_shade_canvas = |p: &mut Painter, tex: Layer, animated: bool| {
			let s = p
				.shade_effect()
				.with_bindings(&[BINDING_BUFFER_FRAG, BINDING_SAMPLER_FRAG])
				.with_layers(&[BINDING_LAYER_FRAG])
				.create();

			let e = p.effect(s).create();
			let layer = p
				.layer()
				.with_effect(e)
				.with_bindings(map! {
					0 => u_size.binding(),
					1 => sampler.binding(),
				})
				.with_layers(map! {
					0 => tex.binding()
				})
				.create();

			(s, Canvas { layer, animated })
		};

		let shade_canvas = |p: &mut Painter, animated: bool| {
			let s = p
				.shade_effect()
				.with_bindings(&[BINDING_BUFFER_FRAG, BINDING_BUFFER_FRAG])
				.create();

			let e = p.effect(s).create();
			let layer = p
				.layer()
				.with_effect(e)
				.with_bindings(map! {
					0 => u_size.binding(),
					1 => u_time.binding()
				})
				.create();

			(s, Canvas { layer, animated })
		};

		let (s, canvas_fbm_shader) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/fbm_shader.spv");

		let (s, canvas_simplex_prefilled) = texture_shade_canvas(p, tex_simplex, false);
		load_fragment_shader!(s, p, "../shader/out/simplex_prefilled.spv");

		let (s, canvas_bos_shaping_fns) = shade_canvas(p, false);
		load_fragment_shader!(s, p, "../shader/out/bos_shaping_fns.spv");

		let (s, canvas_bos_colors) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/bos_colors.spv");

		let (s, canvas_bos_shapes_rect) = shade_canvas(p, false);
		load_fragment_shader!(s, p, "../shader/out/bos_shapes_rect.spv");

		let (s, canvas_bos_shapes_rounded_rect) = shade_canvas(p, false);
		load_fragment_shader!(s, p, "../shader/out/bos_shapes_rounded_rect.spv");

		let (s, canvas_bos_shapes_circle) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/bos_shapes_circle.spv");

		let (s, canvas_bos_shapes_circles) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/bos_shapes_circles.spv");

		let (s, canvas_tiles) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/tiled_plates.spv");

		let (s, canvas_noisy_lines_1) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/noisy_lines_1.spv");

		let (s, canvas_tiled_lines) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/tiled_lines.spv");

		let (s, canvas_noisy_lines_2) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/noisy_lines_2.spv");

		let (s, canvas_noisy_quads) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/noisy_quads.spv");

		let (s, canvas_net) = shade_canvas(p, false);
		load_fragment_shader!(s, p, "../shader/out/net.spv");

		let (s, canvas_pool_tiles) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "../shader/out/pool_tiles.spv");

		// return App

		Self {
			time: 0.0,
			u_size,
			u_time,

			canvases: vec![
				canvas_pool_tiles,
				canvas_noisy_lines_2,
				canvas_tiles,
				canvas_net,
				canvas_tiled_lines,
				canvas_noisy_lines_1,
				canvas_bos_shapes_circles,
				canvas_bos_shapes_circle,
				canvas_bos_shapes_rect,
				canvas_bos_colors,
				canvas_bos_shaping_fns,
				canvas_fbm_shader,
				canvas_simplex_prefilled,
				canvas_bos_shapes_rounded_rect,
				canvas_noisy_quads,
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
			remember_window_dimensions: true,
			features: Some(wgpu::Features::FLOAT32_FILTERABLE),
		})
		.start();
}
