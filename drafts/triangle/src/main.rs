use std::{borrow::Cow, sync::Arc};
use wgpu::{Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::{
	application::ApplicationHandler,
	event::{DeviceEvent, DeviceId, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	window::{Window, WindowId},
};

pub struct RenderState {
	window: Arc<Window>,
	surface: Surface<'static>,
	config: SurfaceConfiguration,
	pipeline: RenderPipeline,
	device: Device,
	queue: Queue,
}

impl RenderState {
	async fn new(window: Arc<Window>) -> Self {
		let mut size = window.inner_size();
		size.width = size.width.max(1);
		size.height = size.height.max(1);

		let instance = wgpu::Instance::default();

		let surface = instance.create_surface(window.clone()).unwrap();
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				force_fallback_adapter: false,
				// Request an adapter which can render to our surface
				compatible_surface: Some(&surface),
			})
			.await
			.expect("Failed to find an appropriate adapter");

		// Create the logical device and command queue
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::empty(),
					// Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
					required_limits: wgpu::Limits::downlevel_webgl2_defaults()
						.using_resolution(adapter.limits()),
					memory_hints: wgpu::MemoryHints::MemoryUsage,
				},
				None,
			)
			.await
			.expect("Failed to create device");

		// Load the shaders from disk
		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});

		let swapchain_capabilities = surface.get_capabilities(&adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: Some("vs_main"),
				buffers: &[],
				compilation_options: Default::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: Some("fs_main"),
				compilation_options: Default::default(),
				targets: &[Some(swapchain_format.into())],
			}),
			primitive: wgpu::PrimitiveState::default(),
			depth_stencil: None,
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
			cache: None,
		});

		let config = surface
			.get_default_config(&adapter, size.width, size.height)
			.unwrap();

		surface.configure(&device, &config);

		Self {
			surface,
			config,
			pipeline,
			device,
			queue,
			window: window.clone(),
		}
	}
}

pub enum WindowState {
	Uninitialized,
	Initializing,
	Initialized(RenderState),
}

pub struct StateInitializationEvent(RenderState);

pub struct Application {
	state: WindowState,
	event_loop_proxy: EventLoopProxy<StateInitializationEvent>,
}

impl Application {
	pub fn new(event_loop: &EventLoop<StateInitializationEvent>) -> Self {
		Self {
			state: WindowState::Uninitialized,
			event_loop_proxy: event_loop.create_proxy(),
		}
	}
}

impl ApplicationHandler<StateInitializationEvent> for Application {
	// This is a common indicator that you can create a window.
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		match self.state {
			WindowState::Initializing | WindowState::Initialized(_) => return,
			WindowState::Uninitialized => {
				self.state = WindowState::Initializing;
			}
		}
		let window = event_loop
			.create_window(Window::default_attributes())
			.unwrap();

		let window = Arc::new(window);

		#[cfg(target_arch = "wasm32")]
		{
			// TODO: initialize canvas
			// web_sys::window()
			// 	.and_then(|win| win.document())
			// 	.and_then(|doc| {
			// 		let dst = doc.get_element_by_id("kloenk-wasm")?;
			// 		let canvas = window.canvas()?;
			// 		canvas
			// 			.set_attribute("tabindex", "0")
			// 			.expect("failed to set tabindex");
			// 		dst.append_child(&canvas).ok()?;
			// 		canvas.focus().expect("Unable to focus on canvas");
			// 		Some(())
			// 	})
			// 	.expect("Couldn't append canvas to document body.");
		}

		let renderer_future = RenderState::new(window);

		#[cfg(target_arch = "wasm32")]
		{
			let event_loop_proxy = self.event_loop_proxy.clone();
			spawn_local(async move {
				let renderer = renderer_future.await;

				event_loop_proxy
					.send_event(StateInitializationEvent(renderer))
					.unwrap_or_else(|_| {
						panic!("Failed to send initialization event");
					});
			});
		}

		#[cfg(not(target_arch = "wasm32"))]
		{
			let renderer = pollster::block_on(renderer_future);

			self
				.event_loop_proxy
				.send_event(StateInitializationEvent(renderer))
				.unwrap_or_else(|_| {
					panic!("Failed to send initialization event");
				});
		}
	}

	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: StateInitializationEvent) {
		let game = event.0;
		game.window.request_redraw();
		self.state = WindowState::Initialized(game);
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		// `unwrap` is fine, the window will always be available when
		// receiving a window event.
		// Handle window event.

		match &mut self.state {
			WindowState::Initialized(renderer) => {
				let window = renderer.window.as_ref();

				match event {
					WindowEvent::Resized(new_size) => {
						// Reconfigure the surface with the new size
						renderer.config.width = new_size.width.max(1);
						renderer.config.height = new_size.height.max(1);
						renderer
							.surface
							.configure(&renderer.device, &renderer.config);
						// On macos the window needs to be redrawn manually after resizing
						window.request_redraw();
					}

					WindowEvent::RedrawRequested => {
						let frame = renderer
							.surface
							.get_current_texture()
							.expect("Failed to acquire next swap chain texture");
						let view = frame
							.texture
							.create_view(&wgpu::TextureViewDescriptor::default());
						let mut encoder = renderer
							.device
							.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
						{
							let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
								label: None,
								color_attachments: &[Some(wgpu::RenderPassColorAttachment {
									view: &view,
									resolve_target: None,
									ops: wgpu::Operations {
										load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
										store: wgpu::StoreOp::Store,
									},
								})],
								depth_stencil_attachment: None,
								timestamp_writes: None,
								occlusion_query_set: None,
							});
							rpass.set_pipeline(&renderer.pipeline);
							rpass.draw(0..3, 0..1);
						}

						renderer.queue.submit(Some(encoder.finish()));
						frame.present();
					}
					WindowEvent::CloseRequested => event_loop.exit(),
					_ => {}
				};
			}
			_ => {}
		}
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: DeviceId,
		_event: DeviceEvent,
	) {
		// Handle window event.
	}
}

pub fn main() {
	let event_loop = EventLoop::<StateInitializationEvent>::with_user_event()
		.build()
		.unwrap();

	#[cfg(not(target_arch = "wasm32"))]
	env_logger::init();

	#[cfg(target_arch = "wasm32")]
	{
		std::panic::set_hook(Box::new(console_error_panic_hook::hook));
		console_log::init().expect("could not initialize logger");
	}

	let mut application = Application::new(&event_loop);
	let _ = event_loop.run_app(&mut application);
}
