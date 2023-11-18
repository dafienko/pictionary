extern crate piston_window;
extern crate image as im;

use std::{sync::{Arc, Mutex}, thread, cmp};
use std::sync::mpsc::{channel, Sender};
use gfx_device_gl::{Factory, Resources, CommandBuffer};

use piston_window::*;
use im::Rgba;

pub enum CanvasOperation {
	Pixel(u32, u32, u8, u8, u8),
	Erase(u32, u32),
	Clear,
}

pub struct GameCanvas {
	pub op_sender: Sender<CanvasOperation>,
	canvas: Arc<Mutex<im::ImageBuffer<Rgba<u8>, Vec<u8>>>>,
	texture_context: TextureContext<Factory, Resources, CommandBuffer>,
	texture: G2dTexture,
}

impl GameCanvas {
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
		
		let (sender, receiver) = channel();
		let c = canvas.clone();
		thread::spawn(move || {
			loop {
				let operation = receiver.recv().unwrap();
				GameCanvas::process_operation(&mut c.lock().unwrap(), width, height, operation);
			}
		});
		
		GameCanvas {
			op_sender: sender,
			canvas: canvas,
			texture_context: texture_context,
			texture: texture
		}
	}

	fn process_operation(c: &mut im::ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32, operation: CanvasOperation) {
		match operation {
			CanvasOperation::Pixel(x, y, r, g, b) => {
				c.put_pixel(x, y, im::Rgba([r, g, b, 255]));
			},

			CanvasOperation::Erase(x, y) => {
				let s = 2;
				for x in cmp::max(0, x - s)..cmp::min(x + s + 1, width) {
					for y in cmp::max(0, y - 1)..cmp::min(y + 2, height) {
						c.put_pixel(x, y, im::Rgba([255, 255, 255, 255]));
					}
				}
			},

			CanvasOperation::Clear => {
				for x in 0..width {
					for y in 0..height {
						c.put_pixel(x, y, im::Rgba([255, 255, 255, 255]));
					}
				}
			}
		}
	}

	pub fn pre_render(&mut self) {
		self.texture.update(&mut self.texture_context, &self.canvas.lock().unwrap()).unwrap();
	}

	pub fn render(&mut self, c: Context, g: &mut G2d, device: &mut gfx_device_gl::Device) {
		self.texture_context.encoder.flush(device);
		
		clear([1.0; 4], g);

		image(&self.texture, c.transform.scale(8.0, 8.0), g);
	}
}