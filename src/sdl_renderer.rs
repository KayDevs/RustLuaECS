use sdl2::{Sdl, VideoSubsystem};
use sdl2::video::{Window, WindowContext};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::rwops::RWops;
use sdl2::pixels::Color;

use std::collections::HashMap;

pub struct SdlRenderer {
	pub screen_width: u32,
	pub screen_height: u32,
	pub video: VideoSubsystem,
	pub canvas: Canvas<Window>,
	pub texture_creator: TextureCreator<WindowContext>,
	pub textures: HashMap<String, Texture>,
}

impl SdlRenderer {
	pub fn new(sdl_context: &Sdl, name: &str, width: u32, height: u32) -> Result<Self, Box<std::error::Error>> {
		let video = sdl_context.video()?;
		video.text_input().stop();
		let window = video.window(name, width, height)
		.position_centered()
		.fullscreen_desktop()
		.build()?;

		//sdl2::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "nearest");

		sdl_context.mouse().show_cursor(false);

		let mut canvas = window.into_canvas()
		.present_vsync()
		.build()?;
		canvas.set_logical_size(640, 400)?;
		canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    	let texture_creator = canvas.texture_creator();
		Ok(SdlRenderer{screen_width: width, screen_height: height, video, canvas, texture_creator, textures: HashMap::new()})

	}

	pub fn insert_texture(&mut self, name: String, tex: &[u8]) {
		let mut surface = Surface::load_bmp_rw(&mut RWops::from_bytes(tex).unwrap()).unwrap();
		surface.set_color_key(true, Color::RGB(255, 0, 255)).unwrap();
		let tex = self.texture_creator.create_texture_from_surface(&surface).unwrap();
		self.textures.insert(name, tex);
	}

	pub fn clear(&mut self, r: u8, g: u8, b: u8) {
        //this makes the letterboxing black on screens with different resolutions
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        //this is the actual background color
        self.canvas.set_draw_color(Color::RGB(r, g, b));
        let _ = self.canvas.fill_rect(None);
	}

	pub fn present(&mut self) {
		self.canvas.present();
	}

    //TODO: text rendering system (leave game logic to lua, only have basic text/textbox drawing)
    /* 
    'text' part of Render:
		- 'font' aliased to 'sprite', draws sprite as font characters
		- 'width' and 'height' are now the width and height of text box, or unbounded if 0
		- 'animation_speed' aliased to 'text_speed', controls typing speed
		- 'text' includes just an ascii string with which to render
		- maybe include ttf support for UTF-8
		- 'border_color', 'border_width', and 'background_color' also settable, default to white on black 
	*/
}


//objects that are 'renderable' can implement this
//there might also be other 'Render' traits for other rendering systems i.e. OpenGL or whatever
pub trait Render {
	fn render(&mut self, renderer: &mut SdlRenderer);
}
