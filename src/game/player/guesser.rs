use piston_window::*;
use crate::game::game_action::GameAction;
use crate::game::communications::Communications;
use crate::game::message::GameMessage;
use crate::game::text_util::{Glyphs, *};
use crate::game::player::{Player, drawer::Drawer, DRAWING_TIME};

pub enum Guesser {
	WaitingForDrawer,
	Guessing(u32, String, String),
	Done(bool, String),
}

impl Guesser {
	pub fn new() -> Self {
		Guesser::WaitingForDrawer
	}
}

impl Player for Guesser {
	fn render(self: &Self, font: &mut Text, glyphs: &mut Glyphs<'_>, c: Context, g: &mut G2d, _device: &mut gfx_device_gl::Device) {
		match self {
			Guesser::WaitingForDrawer => {
				center_text(font, glyphs, "Waiting for Drawer", 400.0, 50.0, c, g);
			},

			Guesser::Guessing(time, skeleton, guess) => {
				let guess: String = skeleton.chars().enumerate().map(|(index, skel_char)| {
					if let Some(guess_char) = guess.chars().nth(index) {
						guess_char
					} else {
						skel_char
					}
				}).collect();

				center_text(font, glyphs, &guess, 400.0, 50.0, c, g);

				font.draw(
					&time.to_string(),
					glyphs,
					&c.draw_state,
					c.transform.trans(10.0, 30.0), g
				).unwrap();
			},

			Guesser::Done(did_win, word) => {
				if *did_win {
					center_text(font, glyphs, "You Win", 400.0, 150.0, c, g);
				} else {
					center_text(font, glyphs, "Time's Up", 400.0, 150.0, c, g);
				}

				center_text(font, glyphs, &format!("'{}'", word), 400.0, 250.0, c, g);
			}
		};
	}

	fn process_action(self: &Self, communications: &mut Communications, action: GameAction) -> Option<Box<dyn Player + Send>> {
		if let GameAction::SwapRoles = action {
			return Some(Box::new(Drawer::new()))
		}

		match self {
			Guesser::WaitingForDrawer => {
				match action {
					GameAction::SetWordSkeleton(skeleton) => {
						Some(Box::new(Guesser::Guessing(DRAWING_TIME, skeleton, "".to_owned())))
					}

					_ => None
				}
			},

			Guesser::Guessing(t, skeleton, guess) => {
				match action {
					GameAction::SetTimeRemaining(time) => {
						Some(Box::new(Guesser::Guessing(time, skeleton.clone(), guess.clone())))
					},

					GameAction::GameOver(word) => {
						Some(Box::new(Guesser::Done(false, word.clone())))
					},

					GameAction::TypeLetter(char) => {
						if guess.len() < skeleton.len() {
							let mut new_guess = guess.clone();
							new_guess.push(char);
							
							Some(Box::new(Guesser::Guessing(*t, skeleton.clone(), new_guess)))
						} else {
							None
						}
					},

					GameAction::DeleteLetter => {
						if guess.len() > 0 {
							let mut new_guess = guess.clone();
							new_guess.pop();
							
							Some(Box::new(Guesser::Guessing(*t, skeleton.clone(), new_guess)))
						} else {
							None
						}
					},

					GameAction::Enter => {
						if guess.len() == skeleton.len() {
							communications.send_message(GameMessage::Guess(guess.clone()));
						}

						None
					},

					GameAction::GuessResult(res) => {
						if let Some(word) = res {
							Some(Box::new(Guesser::Done(true, word.clone())))
						} else {
							Some(Box::new(Guesser::Guessing(*t, skeleton.clone(), "".to_owned())))
						}
					},

					_ => None
				}
			},

			_ => None
		}
	}
}
