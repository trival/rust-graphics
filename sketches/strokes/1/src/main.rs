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
		// Generate painting data
		let painting = create_painting(size.width, size.height, 5);
		let strokes = generate_tile_strokes(&painting);

		let color = calculate_color(painting.tiles.pick().color);
		let col_binding = p.bind_const_vec3(color);

		let (bg_layer, bg_shade) = static_effect_layer_u8(p, 2, 2, map! { 1 => col_binding });
		load_fragment_shader!(bg_shade, p, "../shader/bg_frag.spv");
		let _ = p.init_and_paint(bg_layer);

		// Create line shader for rendering strokes
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

		let u_size = p.bind_const_vec2(vec2(painting.width as f32, painting.height as f32));
		// Create all stroke shapes
		let mut shapes = Vec::new();
		for tile in &strokes {
			let geoms = tile.lines.to_buffered_geometry();
			let rand_offset = p.bind_const_vec2(vec2(rand_f32(), rand_f32()));
			for geom in &geoms {
				let form = p
					.form(geom)
					.with_topology(wgpu::PrimitiveTopology::TriangleStrip)
					.create();
				let color = p.bind_const_vec3(tile.color);
				let shape = p
					.shape(form, line_shade)
					.with_bindings(map! {0 => u_size, 1 => color, 2 => rand_offset})
					.with_blend_state(wgpu::BlendState {
						color: wgpu::BlendComponent {
							src_factor: wgpu::BlendFactor::SrcAlpha,
							dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
							operation: wgpu::BlendOperation::Add,
						},
						alpha: wgpu::BlendComponent {
							src_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
							dst_factor: wgpu::BlendFactor::OneMinusDstAlpha,
							operation: wgpu::BlendOperation::Max,
						},
					})
					.with_cull_mode(Some(wgpu::Face::Back))
					.create();
				shapes.push(shape);
			}
		}

		// Create painting layer with all strokes
		let painting_layer = p
			.layer()
			.with_size(painting.width as u32, painting.height as u32)
			.with_shapes(shapes)
			.with_clear_color(wgpu::Color::TRANSPARENT)
			.create();

		let _ = p.init_and_paint(painting_layer);

		let canvas_shade = p
			.shade_effect()
			.with_bindings(&[BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(canvas_shade, p, "../shader/canvas_frag.spv");

		let sampler = p.sampler_linear();

		let canvas_layer = p
			.single_effect_layer(canvas_shade)
			.with_bindings(map! { 0 => sampler.binding() })
			.with_layers(map! {0 => bg_layer.binding()})
			.with_blend_state(wgpu::BlendState::ALPHA_BLENDING)
			.create();

		let _ = p.init_and_paint(canvas_layer);

		canvas_layer.set_layer_binding(p, 0, painting_layer.binding());

		let _ = p.paint(canvas_layer);

		Self { canvas_layer }
	}

	fn resize(&mut self, p: &mut Painter, _width: u32, _height: u32) {
		p.request_next_frame();
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.show(self.canvas_layer)
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
