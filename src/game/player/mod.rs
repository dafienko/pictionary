
pub mod guesser;
pub mod drawer;
pub mod waiting_player;

use piston_window::*;
use crate::game::game_action::GameAction;
use crate::game::communications::Communications;
use crate::game::text_util::Glyphs;

const DRAWING_TIME: u32 = 100;

pub trait Player {
	fn render(self: &Self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device);

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>>;
}