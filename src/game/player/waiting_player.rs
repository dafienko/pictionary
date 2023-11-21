use piston_window::*;
use crate::game::game_action::GameAction;
use crate::game::communications::Communications;
use crate::game::text_util::{Glyphs, *};
use crate::game::player::Player;

pub struct WaitingPlayer {
	address: String,
}

impl WaitingPlayer {
	pub fn new(address: String) -> Self {
		Self {
			address: address,
		}
	}
}

impl Player for WaitingPlayer {
	fn render(self: &Self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, _device: &mut gfx_device_gl::Device) {
		center_text(font, glyphs, "Waiting for Connection...", 400.0, 150.0, c, g);
		center_text(font, glyphs, &self.address, 400.0, 250.0, c, g);
	}

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>> { 
		match action {
			GameAction::LeftClick(x, y) => {
				communications.send_action(GameAction::Draw(x, y));
				None
			},

			_ => None
		}
	}
}