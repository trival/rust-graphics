use std::collections::BTreeSet;
use trivalibs::{
	painter::{
		app::Event,
		winit::{
			event::{ElementState, KeyEvent, MouseButton, WindowEvent},
			keyboard::{KeyCode, PhysicalKey},
		},
	},
	rendering::camera::PerspectiveCamera,
};

pub struct DraggingState {
	pub start_x: f32,
	pub start_y: f32,
	pub delta_x: f32,
	pub delta_y: f32,
}

pub struct InputState {
	pub pressed_keys: BTreeSet<KeyCode>,
	pub pressed_pointer_buttons: BTreeSet<MouseButton>,
	pub dragging: Option<DraggingState>,
	pub holding: bool,
}

impl Default for InputState {
	fn default() -> Self {
		Self {
			pressed_keys: BTreeSet::new(),
			pressed_pointer_buttons: BTreeSet::new(),
			dragging: None,
			holding: false,
		}
	}
}

impl InputState {
	pub fn process_event<U>(&mut self, event: Event<U>) {
		match event {
			Event::WindowEvent(WindowEvent::KeyboardInput {
				event: KeyEvent {
					state,
					physical_key: PhysicalKey::Code(code),
					..
				},
				..
			}) => {
				if state == ElementState::Pressed {
					self.pressed_keys.insert(code);
				} else {
					self.pressed_keys.remove(&code);
				}
			}

			Event::WindowEvent(WindowEvent::MouseInput { state, button, .. }) => {
				if state == ElementState::Pressed {
					self.pressed_pointer_buttons.insert(button);
				} else {
					self.pressed_pointer_buttons.remove(&button);
				}
			}

			Event::WindowEvent(WindowEvent::CursorMoved { position, .. }) => {
				if self.pressed_pointer_buttons.is_empty() {
					if self.dragging.is_some() {
						self.dragging = None;
					}
				} else {
					if self.dragging.is_none() {
						self.dragging = Some(DraggingState {
							start_x: position.x as f32,
							start_y: position.y as f32,
							delta_x: 0.0,
							delta_y: 0.0,
						});
					} else {
						let dragging = self.dragging.as_mut().unwrap();
						dragging.delta_x = position.x as f32 - dragging.start_x;
						dragging.delta_y = position.y as f32 - dragging.start_y;
					}
				}
			}

			_ => {}
		}
	}
}

pub struct CameraController {
	move_speed: f32,
	look_speed: f32,
	old_drag_x: f32,
	old_drag_y: f32,
}

impl CameraController {
	pub fn new(move_speed: f32, look_speed: f32) -> Self {
		Self {
			move_speed,
			look_speed,
			old_drag_x: 0.0,
			old_drag_y: 0.0,
		}
	}

	pub fn update_camera(
		&mut self,
		camera: &mut PerspectiveCamera,
		input: &InputState,
		delta_time: f32,
	) {
		let mut left = 0.0;
		let mut forward = 0.0;
		let mut rot_x = 0.0;
		let mut rot_y = 0.0;

		let move_distance = self.move_speed * delta_time;

		if input.pressed_keys.contains(&KeyCode::KeyW) || input.pressed_keys.contains(&KeyCode::ArrowUp)
		{
			forward += move_distance;
		}
		if input.pressed_keys.contains(&KeyCode::KeyS)
			|| input.pressed_keys.contains(&KeyCode::ArrowDown)
		{
			forward -= move_distance;
		}
		if input.pressed_keys.contains(&KeyCode::KeyA)
			|| input.pressed_keys.contains(&KeyCode::ArrowLeft)
		{
			left += move_distance;
		}
		if input.pressed_keys.contains(&KeyCode::KeyD)
			|| input.pressed_keys.contains(&KeyCode::ArrowRight)
		{
			left -= move_distance;
		}

		if let Some(dragging) = &input.dragging {
			let delta_x = self.old_drag_x - dragging.delta_x;
			let delta_y = self.old_drag_y - dragging.delta_y;

			self.old_drag_x = dragging.delta_x;
			self.old_drag_y = dragging.delta_y;

			rot_x += delta_x * self.look_speed * 0.001;
			rot_y += delta_y * self.look_speed * 0.001;
		} else {
			self.old_drag_x = 0.0;
			self.old_drag_y = 0.0;
		}

		if left != 0.0 || forward != 0.0 || rot_x != 0.0 || rot_y != 0.0 {
			camera.update_transform(forward, left, 0.0, rot_x, rot_y);
		}
	}
}
