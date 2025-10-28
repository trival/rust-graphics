use trivalibs::{
	map, painter::prelude::*, prelude::*,
	rendering::line_2d::buffered_geometry::LineBufferedGeometryVec,
};

mod painting;
use painting::{create_painting, generate_tile_strokes};

struct App {
	painting_layer: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let size = p.canvas_size();
		// Generate painting data
		let painting = create_painting(size.width, size.height, 5);
		let strokes = generate_tile_strokes(&painting);

		// TODO: Use tile colors for the strokes in the future

		// Create line shader for rendering strokes
		let line_shade = p
			.shade(&[Float32x2, Float32, Float32, Float32x2, Float32x2])
			.with_bindings(&[BINDING_BUFFER_VERT, BINDING_BUFFER_FRAG])
			.create();
		load_vertex_shader!(line_shade, p, "../shader/line_vert.spv");
		load_fragment_shader!(line_shade, p, "../shader/line_frag.spv");

		let u_size = p.bind_const_vec2(vec2(painting.width as f32, painting.height as f32));
		// Create all stroke shapes
		let mut shapes = Vec::new();
		for tile in &strokes {
			let geoms = tile.lines.to_buffered_geometry();
			for geom in &geoms {
				let form = p
					.form(geom)
					.with_topology(wgpu::PrimitiveTopology::TriangleStrip)
					.create();
				let color = p.bind_const_vec3(tile.color);
				let shape = p
					.shape(form, line_shade)
					.with_bindings(map! {0 => u_size, 1 => color})
					.create();
				shapes.push(shape);
			}
		}

		// Create background shader
		let bg_shade = p
			.shade_effect()
			.with_bindings(&[BINDING_BUFFER_FRAG])
			.create();
		load_fragment_shader!(bg_shade, p, "../shader/bg_frag.spv");

		let bg_effect = p.effect(bg_shade).create();

		let color = p.bind_const_vec3(vec3(0.1, 0.9, 0.9));

		// Create background layer
		let background_layer = p
			.layer()
			.with_size(2, 2)
			.with_effect(bg_effect)
			.with_bindings(map! {0 => color})
			.create_and_init();

		let _ = p.paint(background_layer);

		// Create painting layer with all strokes
		let painting_layer = p
			.layer()
			.with_size(painting.width as u32, painting.height as u32)
			.with_shapes(shapes)
			.with_clear_color(wgpu::Color::TRANSPARENT)
			.create();

		Self { painting_layer }
	}

	fn resize(&mut self, p: &mut Painter, _width: u32, _height: u32) {
		p.request_next_frame();
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint_and_show(self.painting_layer)
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
