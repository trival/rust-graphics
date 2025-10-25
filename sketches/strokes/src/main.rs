use trivalibs::{painter::prelude::*, prelude::*, rendering::BufferedGeometry};

mod painting;
use painting::{create_painting, generate_tile_strokes};

struct App {
	painting_layer: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		// Generate painting data
		let painting = create_painting(1200, 1200, 5);
		let strokes = generate_tile_strokes(&painting);

		// TODO: Use tile colors for the strokes in the future

		// Create line shader for rendering strokes
		let line_shade = p
			.shade(&[Float32x2, Float32, Float32, Float32x2, Float32x2])
			.create();
		load_vertex_shader!(line_shade, p, "../shader/line_vert.spv");
		load_fragment_shader!(line_shade, p, "../shader/line_frag.spv");

		// Create all stroke shapes
		let mut shapes = Vec::new();
		for tile in &strokes {
			for line in &tile.lines {
				let geom: BufferedGeometry = line.to_buffered_geometry();
				let form = p
					.form(&geom)
					.with_topology(wgpu::PrimitiveTopology::TriangleStrip)
					.create();
				let shape = p.shape(form, line_shade).create();
				shapes.push(shape);
			}
		}

		// Create background shader
		let bg_shade = p.shade_effect().create();
		load_fragment_shader!(bg_shade, p, "../shader/bg_frag.spv");

		let bg_effect = p.effect(bg_shade).create();

		// Create background layer
		let background_layer = p
			.layer()
			.with_size(painting.width as u32, painting.height as u32)
			.with_effect(bg_effect)
			.create_and_init();

		let _ = p.paint(background_layer);

		// Create painting layer with all strokes
		let painting_layer = p
			.layer()
			.with_size(painting.width as u32, painting.height as u32)
			.with_shapes(shapes)
			.with_clear_color(wgpu::Color::TRANSPARENT)
			.create_and_init();

		// Render the painting once during init
		let _ = p.paint(painting_layer);

		Self { painting_layer }
	}

	fn resize(&mut self, _p: &mut Painter, _width: u32, _height: u32) {}

	fn update(&mut self, p: &mut Painter, _tpf: f32) {
		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		// Show the painting
		p.show(self.painting_layer)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: false,
			keep_window_dimensions: true,
			..default()
		})
		.start();
}
