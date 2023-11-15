/*

game = connect_players()

init window

message pump {
	if let action {
		game.process_action(action)
	}

	game.render()
}

*/


pub enum GameAction {
	Tick(),
	TypeLetter(u8),
	DeleteLetter(),
	Draw(u32, u32),
	Enter(),
}


trait Player {
	fn render(self: &Self);
	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player>>;
}

enum DrawerState {
	Init,
	PickingWord(Vec<String>),
	Drawing(String, u32)
}

enum GuesserState {
	Init,
	WaitingForDrawer,
	Guessing(String, String)
}

pub struct Drawer {
	state: DrawerState
}

impl Player for Drawer {
	fn render(self: &Self) {

	}

	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player>> {

		None
	}
}
	
pub struct Guesser {
	state: GuesserState
}

impl Player for Guesser {
	fn render(self: &Self) {

	}

	fn process_action(self: &Self, action: GameAction) -> Option<Box<dyn Player>> {

		None
	}
}

pub struct Game {
	role: Box<dyn Player>,
}

impl Game {
	pub fn render(&self) {
		self.role.render();
	}

	pub fn process_action(&mut self, action: GameAction) {
		if let Some(new) = self.role.process_action(action) {
			self.role = new;
		};
	}
}

