extern crate piston_window;
extern crate image as im;

use piston_window::*;
use gfx_device_gl::{Factory, Resources, CommandBuffer};
use piston_window::glyph_cache::rusttype::GlyphCache;

pub type Glyphs<'a> = GlyphCache<'a, TextureContext<Factory, Resources, CommandBuffer>, Texture<Resources>>;

pub fn metrics<C>(
	obj: &Text,
	text: &str,
	cache: &mut C
) -> f64 where C: CharacterCache {
	text.chars().fold(0.0, |sum, char| {
		sum + cache.character(obj.font_size, char).unwrap().advance_width()
	})
}

pub fn center_text(font: &mut Text, glyphs: &mut Glyphs<'_>, text: &str, x: f64, y: f64, c: Context, g: &mut G2d) {
	let w = metrics(font, text, glyphs);
	font.draw(
		text,
		glyphs,
		&c.draw_state,
		c.transform.trans(x - w * 0.5, y), g
	).unwrap();
}