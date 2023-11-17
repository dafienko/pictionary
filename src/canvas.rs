extern crate piston_window;
extern crate image as im;

#[path = "text_util.rs"] mod text_util;

use std::{sync::{Arc, Mutex}, thread};
use std::sync::mpsc::{channel, Sender};

use gfx_device_gl::{Factory, Resources, CommandBuffer};
use piston_window::*;
use im::Rgba;
use piston_window::glyph_cache::rusttype::GlyphCache;
use text_util::metrics;

pub enum CanvasOperation {
	Pixel(u32, u32, u8, u8, u8),
}

pub struct GameCanvas<'a> {
	op_sender: Sender<CanvasOperation>,
	canvas: Arc<Mutex<im::ImageBuffer<Rgba<u8>, Vec<u8>>>>,
	texture_context: TextureContext<Factory, Resources, CommandBuffer>,
	texture: G2dTexture,
	font: Text,
	glyphs: GlyphCache<'a, TextureContext<Factory, Resources, CommandBuffer>, Texture<Resources>>
}

impl<'a> GameCanvas<'a> {
	pub fn new(window: &mut PistonWindow, width: u32, height: u32) -> Self {
		let canvas = Arc::new(Mutex::new(
			im::ImageBuffer::new(width, height)
		));
	
		let mut texture_context = TextureContext {
			factory: window.factory.clone(),
			encoder: window.factory.create_command_buffer().into()
		};

		let texture = Texture::from_image(
			&mut texture_context,
			&canvas.lock().unwrap(),
			&TextureSettings::new().filter(Filter::Nearest)
		).unwrap();

		let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
		let glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();
		
		let (sender, receiver) = channel();
		let c = canvas.clone();
		thread::spawn(move || {
			loop {
				let operation = receiver.recv().unwrap();
				GameCanvas::process_operation(&mut c.lock().unwrap(), operation);
			}
		});
		
		GameCanvas {
			op_sender: sender,
			canvas: canvas,
			texture_context: texture_context,
			texture: texture,
			font: text::Text::new_color([0.0, 0.0, 0.0, 1.0], 32),
			glyphs: glyphs
		}
	}

	fn process_operation(c: &mut im::ImageBuffer<Rgba<u8>, Vec<u8>>, operation: CanvasOperation) {
		match operation {
			CanvasOperation::Pixel(x, y, r, g, b) => {
				c.put_pixel(x, y, im::Rgba([r, g, b, 255]));
			}
		}
	}

	pub fn pre_render(&mut self) {
		self.texture.update(&mut self.texture_context, &self.canvas.lock().unwrap()).unwrap();
	}

	pub fn render(&mut self, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		self.texture_context.encoder.flush(device);
		self.glyphs.factory.encoder.flush(device);
		
		clear([1.0; 4], g);

		image(&self.texture, c.transform.scale(4.0, 4.0), g);
	}

	pub fn push(&mut self, operation: CanvasOperation) {
		self.op_sender.send(operation).unwrap();
	}

	pub fn center_text(&mut self, text: &str, x: f64, y: f64, c: Context, g: &mut G2d) {
		let w = metrics(&self.font, text, &mut self.glyphs);
		self.font.draw(
			text,
			&mut self.glyphs,
			&c.draw_state,
			c.transform.trans(x - w * 0.5, y), g
		).unwrap();
	}

}