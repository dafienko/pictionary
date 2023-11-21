#[derive(Debug)]
pub enum GameAction {
	Update(f64),
	Tick,
	TypeNumber(u8),
	TypeLetter(char),
	DeleteLetter,
	Enter,
	SetTimeRemaining(u32),
	SetWordSkeleton(String),
	Draw(u32, u32),
	Erase(u32, u32),
	LeftClick(u32, u32),
	RightClick(u32, u32),
	Guess(String),
	GuessResult(Option<String>),
	GameOver(String),
	SwapRoles,
}