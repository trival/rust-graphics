use trivalibs::{map, painter::prelude::*, prelude::*};

struct App {
	time: f32,

	// Bindings
	u_time: BindingBuffer<f32>,
	u_threshold: BindingBuffer<f32>,
	u_bloom_intensity: BindingBuffer<f32>,
	u_resolution_mip0: BindingBuffer<Vec2>,
	u_resolution_mip1: BindingBuffer<Vec2>,
	u_resolution_mip2: BindingBuffer<Vec2>,
	u_resolution_mip3: BindingBuffer<Vec2>,

	// Layers
	scene_layer: Layer,
	bloom_layer: Layer,
	canvas: Layer,
}

impl CanvasApp for App {
	fn init(p: &mut Painter) -> Self {
		// Create shaders
		let test_scene_shade = p
			.shade_effect()
			.with_bindings([BINDING_BUFFER_FRAG])
			.create();
		load_fragment_shader!(test_scene_shade, p, "./shader/test_scene_fs.spv");

		let threshold_shade = p
			.shade_effect()
			.with_bindings([BINDING_BUFFER_FRAG, BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(threshold_shade, p, "./shader/threshold_fs.spv");

		let downsample_shade = p
			.shade_effect()
			.with_bindings([BINDING_BUFFER_FRAG, BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(downsample_shade, p, "./shader/downsample_blur_fs.spv");

		let upsample_shade = p
			.shade_effect()
			.with_bindings([BINDING_BUFFER_FRAG, BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(upsample_shade, p, "./shader/upsample_blur_fs.spv");

		let composite_shade = p
			.shade_effect()
			.with_bindings([BINDING_BUFFER_FRAG, BINDING_SAMPLER_FRAG])
			.with_layers([BINDING_LAYER_FRAG, BINDING_LAYER_FRAG])
			.create();
		load_fragment_shader!(composite_shade, p, "./shader/composite_fs.spv");

		// Create bindings
		let u_time = p.bind_f32();
		let u_threshold = p.bind_f32();
		let u_bloom_intensity = p.bind_f32();
		let u_resolution_mip0 = p.bind_vec2();
		let u_resolution_mip1 = p.bind_vec2();
		let u_resolution_mip2 = p.bind_vec2();
		let u_resolution_mip3 = p.bind_vec2();

		// Initialize parameter values
		u_threshold.update(p, 1.0);
		u_bloom_intensity.update(p, 0.8);

		// Create sampler
		let sampler = p
			.sampler()
			.with_filters(wgpu::FilterMode::Linear)
			.with_mipmap_filter(wgpu::FilterMode::Linear)
			.create();

		// Create scene layer using single_effect_layer
		let scene_layer = p
			.single_effect_layer(test_scene_shade)
			.with_bindings(vec![(0, u_time.binding())])
			.create();

		// Create threshold effect
		let threshold_effect = p
			.effect(threshold_shade)
			.with_bindings(map! {
					0 => u_threshold.binding(),
					1 => sampler.binding(),
			})
			.with_layers(map! {
					0 => scene_layer.binding()
			})
			.with_mip_target(0)
			.create();

		// Create additive blend state for upsampling
		let additive_blend = wgpu::BlendState {
			color: wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::One,
				operation: wgpu::BlendOperation::Add,
			},
			alpha: wgpu::BlendComponent::REPLACE,
		};

		let mut effects = vec![threshold_effect];

		// Downsample chain: mip 0→1→2→3→4 (single pass per level)
		for i in 0..4 {
			let res_binding = match i {
				0 => u_resolution_mip1.binding(),
				1 => u_resolution_mip2.binding(),
				2 => u_resolution_mip3.binding(),
				3 => u_resolution_mip3.binding(), // mip 4 has same calculation
				_ => unreachable!(),
			};

			effects.push(
				p.effect(downsample_shade)
					.with_bindings(map! {
							0 => res_binding,
							1 => sampler.binding(),
					})
					.with_mip_source(i)
					.with_mip_target(i + 1)
					.create(),
			);
		}

		// Upsample chain: mip 4→3→2→1→0 (single pass per level with additive blending)
		for i in (0..4).rev() {
			let res_binding = match i {
				0 => u_resolution_mip0.binding(),
				1 => u_resolution_mip1.binding(),
				2 => u_resolution_mip2.binding(),
				3 => u_resolution_mip3.binding(),
				_ => unreachable!(),
			};

			effects.push(
				p.effect(upsample_shade)
					.with_bindings(map! {
							0 => res_binding,
							1 => sampler.binding(),
					})
					.with_mip_source(i + 1)
					.with_mip_target(i)
					.with_blend_state(additive_blend)
					.create(),
			);
		}

		// Create bloom layer with all effects
		let bloom_layer = p.layer().with_effects(effects).with_mips_max(5).create();

		// Create final composite layer
		let canvas = p
			.single_effect_layer(composite_shade)
			.with_bindings(vec![
				(0, u_bloom_intensity.binding()),
				(1, sampler.binding()),
			])
			.with_layers(vec![(0, scene_layer.binding()), (1, bloom_layer.binding())])
			.create();

		Self {
			time: 0.0,
			u_time,
			u_threshold,
			u_bloom_intensity,
			u_resolution_mip0,
			u_resolution_mip1,
			u_resolution_mip2,
			u_resolution_mip3,

			scene_layer,
			bloom_layer,
			canvas,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self
			.u_resolution_mip0
			.update(p, vec2(width as f32, height as f32));
		self
			.u_resolution_mip1
			.update(p, vec2((width / 2) as f32, (height / 2) as f32));
		self
			.u_resolution_mip2
			.update(p, vec2((width / 4) as f32, (height / 4) as f32));
		self
			.u_resolution_mip3
			.update(p, vec2((width / 8) as f32, (height / 8) as f32));
	}

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);

		p.paint_and_show(self.scene_layer);
		// p.paint(self.bloom_layer);
		// p.paint_and_show(self.canvas);

		p.request_next_frame();
	}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: false,
			remember_window_dimensions: true,
			..default()
		})
		.start();
}
