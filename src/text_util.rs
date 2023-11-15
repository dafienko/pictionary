extern crate piston_window;
use piston_window::*;

pub fn metrics<C>(
	obj: &Text,
	text: &str,
	cache: &mut C
) -> f64 where C: CharacterCache {
	text.chars().fold(0.0, |sum, char| {
		sum + cache.character(obj.font_size, char).unwrap().advance_width()
	})
}