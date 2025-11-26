use shared::static_effect_layer_u8;
use trivalibs::{
	map, painter::prelude::*, prelude::*,
	rendering::line_2d::buffered_geometry::LineBufferedGeometryVec,
};

mod painting;
use painting::{create_painting, generate_tile_strokes};

use crate::painting::calculate_color;

struct App {
	canvas_layer: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let size = p.canvas_size();

		let painting = create_painting(size.width, size.height, 5);

		let bg_color = calculate_color(painting.tiles.pick().color);

		let u_color = p.bind_vec3();
		u_color.update_vec3(p, bg_color);

		let (bg_layer, bg_shade) = static_effect_layer_u8(p, 2, 2, map! { 1 => u_color.binding() });
		load_fragment_shader!(bg_shade, p, "../shader/bg_frag.spv");
		p.init_and_paint(bg_layer);

		let line_shade = p
			.shade(&[Float32x2, Float32, Float32, Float32x2, Float32x2])
			.with_bindings(&[
				BINDING_BUFFER_VERT,
				BINDING_BUFFER_FRAG,
				BINDING_BUFFER_FRAG,
			])
			.create();
		load_vertex_shader!(line_shade, p, "../shader/line_vert.spv");
		load_fragment_shader!(line_shade, p, "../shader/line_frag.spv");

		let line_form = p
			.form_builder()
			.with_topology(wgpu::PrimitiveTopology::TriangleStrip)
			.create();

		let blend_state = wgpu::BlendState {
			color: wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::Zero,
				operation: wgpu::BlendOperation::Add,
			},
			alpha: wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::One,
				operation: wgpu::BlendOperation::Max,
			},
		};

		let u_size = p.bind_const_vec2(vec2(painting.width as f32, painting.height as f32));
		let u_rand_offset = p.bind_vec2();

		let line_shape = p
			.shape(line_form, line_shade)
			.with_blend_state(blend_state)
			.with_bindings(map! {
				0 => u_size,
				1 => u_color.binding(),
				2 => u_rand_offset.binding()
			})
			.create();

		let painting_layer = p
			.layer()
			.with_size(painting.width as u32, painting.height as u32)
			.with_shape(line_shape)
			.with_clear_color(wgpu::Color::TRANSPARENT)
			.create_and_init();

		let canvas_shade = p
			.shade_effect()
			.with_bindings(&[BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(canvas_shade, p, "../shader/canvas_frag.spv");

		let sampler = p.sampler_linear();

		let canvas_layer = p
			.single_effect_layer(canvas_shade)
			.with_size(painting.width, painting.height)
			.with_bindings(map! { 0 => sampler.binding() })
			.with_layers(map! {0 => bg_layer.binding()})
			.with_blend_state(wgpu::BlendState::ALPHA_BLENDING)
			.create_and_init();

		canvas_layer.set_layer_binding(p, 0, bg_layer.binding());

		p.paint(canvas_layer);

		canvas_layer.set_layer_binding(p, 0, painting_layer.binding());

		// === painting process ===

		let paint_strokes = |p: &mut Painter| {
			let strokes = generate_tile_strokes(&painting);

			for tile in &strokes {
				line_form.update_all(p, &tile.lines.to_buffered_geometry());

				u_rand_offset.update(p, vec2(rand_f32(), rand_f32()));
				u_color.update_vec3(p, tile.color);

				p.compose(&[painting_layer, canvas_layer]);
			}
		};

		paint_strokes(p);
		paint_strokes(p);
		paint_strokes(p);

		Self { canvas_layer }
	}

	fn resize(&mut self, p: &mut Painter, _width: u32, _height: u32) {
		p.request_next_frame();
	}

	fn frame(&mut self, p: &mut Painter, _tpf: f32) {
		p.show(self.canvas_layer);
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
