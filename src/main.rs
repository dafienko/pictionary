extern crate piston_window;
extern crate image as im;

mod game;
mod message;
mod canvas;

use piston_window::*;
use crate::canvas::GameCanvas;
use crate::game::Game;

fn main() {
	let size = 100;
    let mut window: PistonWindow = WindowSettings::new(
		"piston: hello_world",
		[size * 8; 2]
	)
	.exit_on_esc(true)
	.graphics_api(OpenGL::V4_1)
	.resizable(false)
	.build()
	.unwrap();

	let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
	let mut glyphs = window.load_font(assets.join("FiraSans-Regular.ttf")).unwrap();
	let mut font = text::Text::new_color([0.0, 0.0, 0.0, 1.0], 32);

	let mut canvas = GameCanvas::new(&mut window, size, size);
	
	let game = Game::new(canvas.op_sender.clone());

    while let Some(e) = window.next() {
        if e.render_args().is_some() {
			canvas.pre_render();
			
            window.draw_2d(&e, |c, g, device| {
				glyphs.factory.encoder.flush(device);
				canvas.render(c, g, device);
                game.lock().unwrap().render(&mut font, &mut glyphs, c, g, device);				
            });
        }

		game.lock().unwrap().process_event(e)
    }
}