use macroquad::{input::{is_mouse_button_released, mouse_position, MouseButton}, math::Vec2, ui::{root_ui, widgets}};

pub struct Button {
	label: String,
	position: Vec2,
	size: Vec2
}

impl Button {
	pub fn new(label: &str, position: Vec2, size: Vec2) -> Self {
		Self {
			label: label.to_string(),
			position,
			size,
		}
	}

	pub fn draw(&self) {
		widgets::Button::new(self.label.as_str())
			.position(self.position)
			.size(self.size)
			.ui(&mut root_ui());
	}

	pub fn was_clicked(&self) -> bool {
		if !is_mouse_button_released(MouseButton::Left) {
			return false;
		}
		
		let mouse_position = mouse_position();
		let x = mouse_position.0;
		let y = mouse_position.1;
		let x1 = self.position.x;
		let x2 = self.position.x + self.size.x;
		let y1 = self.position.y;
		let y2 = self.position.y + self.size.y;
		x >= x1 && x <= x2 && y >= y1 && y <= y2
	}
}