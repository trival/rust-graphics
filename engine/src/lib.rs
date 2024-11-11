use std::sync::Arc;
use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use winit::{
	application::ApplicationHandler,
	event::{DeviceEvent, DeviceId, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	window::{Window, WindowId},
};

pub struct Renderer {
	pub window: Arc<Window>,
	pub surface: Surface<'static>,
	pub adapter: Adapter,
	pub config: SurfaceConfiguration,
	pub device: Device,
	pub queue: Queue,
}

impl Renderer {
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

		let config = surface
			.get_default_config(&adapter, size.width, size.height)
			.unwrap();

		surface.configure(&device, &config);

		Self {
			surface,
			config,
			adapter,
			device,
			queue,
			window: window.clone(),
		}
	}
}

pub trait Application<UserEvent> {
	fn init(&mut self, renderer: &Renderer);
	fn render(&self, renderer: &Renderer);
	fn window_event(&mut self, event: winit::event::WindowEvent);
	fn device_event(&mut self, event: winit::event::DeviceEvent);
	fn user_event(&mut self, event: UserEvent);
}

enum WindowState {
	Uninitialized,
	Initializing,
	Initialized(Renderer),
}

pub enum CustomEvent<UserEvent> {
	StateInitializationEvent(Renderer),
	UserEvent(UserEvent),
}

pub struct ApplicationRunner<UserEvent, App>
where
	UserEvent: 'static,
	App: Application<UserEvent>,
{
	state: WindowState,
	event_loop_proxy: EventLoopProxy<CustomEvent<UserEvent>>,
	app: App,
}

impl<UserEvent, App> ApplicationRunner<UserEvent, App>
where
	App: Application<UserEvent>,
{
	pub fn start(app: App) {
		#[cfg(not(target_arch = "wasm32"))]
		env_logger::init();

		#[cfg(target_arch = "wasm32")]
		{
			std::panic::set_hook(Box::new(console_error_panic_hook::hook));
			console_log::init().expect("could not initialize logger");
		}

		let event_loop = EventLoop::<CustomEvent<UserEvent>>::with_user_event()
			.build()
			.unwrap();

		let event_loop_proxy = event_loop.create_proxy();

		let mut runner = Self {
			state: WindowState::Uninitialized,
			event_loop_proxy,
			app,
		};

		let _ = event_loop.run_app(&mut runner);
	}
}

impl<UserEvent, App> ApplicationHandler<CustomEvent<UserEvent>>
	for ApplicationRunner<UserEvent, App>
where
	App: Application<UserEvent>,
{
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

		let renderer_future = Renderer::new(window);

		#[cfg(target_arch = "wasm32")]
		{
			let event_loop_proxy = self.event_loop_proxy.clone();
			spawn_local(async move {
				let renderer = renderer_future.await;

				event_loop_proxy
					.send_event(CustomEvent(renderer))
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
				.send_event(CustomEvent::StateInitializationEvent(renderer))
				.unwrap_or_else(|_| {
					panic!("Failed to send initialization event");
				});
		}
	}

	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: CustomEvent<UserEvent>) {
		match event {
			CustomEvent::StateInitializationEvent(renderer) => {
				renderer.window.request_redraw();
				self.app.init(&renderer);
				self.state = WindowState::Initialized(renderer);
			}
			CustomEvent::UserEvent(user_event) => {
				self.app.user_event(user_event);
			}
		}
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
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
						self.app.render(renderer);
					}

					WindowEvent::CloseRequested => event_loop.exit(),
					rest => {
						self.app.window_event(rest);
					}
				};
			}
			_ => {}
		}
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: DeviceId,
		event: DeviceEvent,
	) {
		self.app.device_event(event);
	}
}
