use trivalibs::{
	bmap,
	painter::{
		create_canvas_app,
		effect::EffectProps,
		layer::{Layer, LayerProps},
		shade::ShadeEffectProps,
		uniform::UniformBuffer,
		CanvasApp, Painter,
	},
	prelude::*,
	wgpu::{include_spirv, ShaderStages, SurfaceError},
	winit::event::{DeviceEvent, WindowEvent},
};

struct RenderState {
	time: UniformBuffer<f32>,
	size: UniformBuffer<UVec2>,
	canvas: Layer,
}

#[derive(Default)]
struct App {
	time: f32,
}

impl CanvasApp<RenderState, ()> for App {
	fn init(&mut self, p: &mut Painter) -> RenderState {
		let uniform_layout = p.uniform_get_layout_buffered(ShaderStages::FRAGMENT);

		let shade = p.shade_create_effect(ShadeEffectProps {
			shader: include_spirv!("../shader/main.spv"),
			uniform_layout: &[&uniform_layout, &uniform_layout],
		});

		let time = p.uniform_create(&uniform_layout, 0.0f32);
		let size = p.uniform_create(&uniform_layout, uvec2(0, 0));

		let effect = p.effect_create(
			shade,
			&EffectProps {
				uniforms: bmap! {
					0 => size.uniform,
					1 => time.uniform,
				},
				..default()
			},
		);

		let canvas = p.layer_create(&LayerProps {
			effects: vec![effect],
			..default()
		});

		RenderState { canvas, time, size }
	}

	fn resize(&mut self, p: &mut Painter, rs: &mut RenderState) {
		let size = p.canvas_size();
		rs.size.update(p, uvec2(size.width, size.height));
	}

	fn update(&mut self, p: &mut Painter, rs: &mut RenderState, tpf: f32) {
		self.time += tpf;
		rs.time.update(p, self.time);
	}

	fn render(&self, p: &mut Painter, state: &RenderState) -> Result<(), SurfaceError> {
		p.paint(&state.canvas)?;
		p.show(&state.canvas)?;

		p.request_next_frame();

		Ok(())
	}

	fn user_event(&mut self, _e: (), _p: &Painter) {}
	fn window_event(&mut self, _e: WindowEvent, _p: &Painter) {}
	fn device_event(&mut self, _e: DeviceEvent, _p: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
