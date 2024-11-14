use std::sync::Arc;
use wgpu::SurfaceError;
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::{DeviceEvent, DeviceId, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	window::{Window, WindowId},
};

mod renderer;
pub use renderer::Renderer;

pub trait Application<UserEvent> {
	fn init(&mut self, renderer: &Renderer);
	fn render(&self, renderer: &Renderer) -> Result<(), SurfaceError>;
	fn window_event(&mut self, event: WindowEvent, renderer: &Renderer);
	fn device_event(&mut self, event: DeviceEvent, renderer: &Renderer);
	fn user_event(&mut self, event: UserEvent, renderer: &Renderer);
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

pub struct ApplicationHandle<UserEvent>
where
	UserEvent: 'static,
{
	event_loop_proxy: EventLoopProxy<CustomEvent<UserEvent>>,
}

impl<UserEvent> ApplicationHandle<UserEvent> {
	pub fn send_event(
		&self,
		event: UserEvent,
	) -> Result<(), winit::event_loop::EventLoopClosed<CustomEvent<UserEvent>>> {
		self
			.event_loop_proxy
			.send_event(CustomEvent::UserEvent(event))
	}
}

pub struct ApplicationStarter<UserEvent, App>
where
	UserEvent: 'static,
	App: Application<UserEvent>,
{
	app: ApplicationRunner<UserEvent, App>,
	event_loop: EventLoop<CustomEvent<UserEvent>>,
}

impl<UserEvent, App> ApplicationStarter<UserEvent, App>
where
	UserEvent: std::marker::Send,
	App: Application<UserEvent> + std::marker::Send + 'static,
{
	pub fn start(self) {
		let event_loop = self.event_loop;
		let mut app = self.app;
		let _ = event_loop.run_app(&mut app);
	}

	pub fn get_handle(&self) -> ApplicationHandle<UserEvent> {
		ApplicationHandle {
			event_loop_proxy: self.app.event_loop_proxy.clone(),
		}
	}
}

pub fn create_app<UserEvent, App: Application<UserEvent> + 'static>(
	app: App,
) -> ApplicationStarter<UserEvent, App> {
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

	let runner = ApplicationRunner {
		state: WindowState::Uninitialized,
		event_loop_proxy,
		app,
	};

	return ApplicationStarter {
		app: runner,
		event_loop,
	};
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
				if let WindowState::Initialized(renderer) = &self.state {
					self.app.user_event(user_event, renderer);
				}
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
				match event {
					WindowEvent::Resized(new_size) => {
						// Reconfigure the surface with the new size
						renderer.resize(new_size);
						// On macos the window needs to be redrawn manually after resizing
						renderer.request_redraw();
					}

					WindowEvent::RedrawRequested => {
						match self.app.render(renderer) {
							Ok(_) => {}
							// Reconfigure the surface if it's lost or outdated
							Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
								renderer.resize(PhysicalSize {
									width: renderer.config.width,
									height: renderer.config.height,
								});
							}
							// The system is out of memory, we should probably quit
							Err(wgpu::SurfaceError::OutOfMemory) => {
								log::error!("OutOfMemory");
								event_loop.exit();
							}

							// This happens when the a frame takes too long to present
							Err(wgpu::SurfaceError::Timeout) => {
								log::warn!("Surface timeout")
							}
						}
					}

					WindowEvent::CloseRequested => event_loop.exit(),
					rest => {
						self.app.window_event(rest, renderer);
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
		if let WindowState::Initialized(renderer) = &mut self.state {
			self.app.device_event(event, renderer);
		}
	}
}
