extern crate piston_window;
extern crate image as im;
#[path = "ts_dequeue.rs"] mod ts_dequeue;

use std::sync::{Arc, Mutex};

use gfx_device_gl::{Factory, Resources, CommandBuffer};
use piston_window::*;
use im::Rgba;
use ts_dequeue::TSDequeue;
use piston_window::glyph_cache::rusttype::GlyphCache;

enum CanvasOperation {
	Pixel(u32, u32, u8, u8, u8),
}

pub struct GameCanvas<'a> {
	op_queue: Arc<TSDequeue<CanvasOperation>>,
	canvas: Arc<Mutex<im::ImageBuffer<Rgba<u8>, Vec<u8>>>>,
	texture_context: TextureContext<Factory, Resources, CommandBuffer>,
	texture: G2dTexture,
	font: Text,
	glyphs: GlyphCache<'a, TextureContext<Factory, Resources, CommandBuffer>, Texture<Resources>>
}

impl GameCanvas<'_> {
	pub fn new(window: &PistonWindow, width: u32, height: u32) -> Self {
		let canvas = Arc::new(Mutex::new(
			im::ImageBuffer::new(width, height)
		));
	
		let mut texture_context = TextureContext {
			factory: window.factory.clone(),
			encoder: window.factory.create_command_buffer().into()
		};

		let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
		let mut glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();

		GameCanvas {
			op_queue: Arc::new(TSDequeue::<CanvasOperation>::new()),
			canvas: canvas,
			texture_context: texture_context,
			texture: Texture::from_image(
				&mut texture_context,
				&canvas.lock().unwrap(),
				&TextureSettings::new().filter(Filter::Nearest)
			).unwrap(),
			font: text::Text::new_color([0.0, 0.0, 0.0, 1.0], 32),
			glyphs: glyphs
		}
	}

	fn process_operation(c: &mut Mutex<im::ImageBuffer<Rgba<u8>, Vec<u8>>>, operation: CanvasOperation) {
		match operation {
			CanvasOperation::Pixel(x, y, r, g, b) => {
				c.lock().unwrap().put_pixel(x, y, im::Rgba([r, g, b, 255]));
			}
		}
	}

	pub fn process_operation_queue(&self) {
		let mut c = self.canvas.clone();
		let q = self.op_queue.clone();
		loop {
			if !q.is_empty() {
				let operation = q.pop();
				GameCanvas::process_operation(&mut c, operation);
			}
		}
	}
}