use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

//this one's not a NativeSystem it's just a thing
//this could probably be more sophisticated but it works?
pub struct Terminal {
	commands: Vec<String>,
	outputs: Vec<String>,
	commandline: String,
	height: u32, //max height
	on: bool,
	h: u32,		//actual height within animation
	char_width: u32,
	char_height: u32,
}

impl Terminal {
	pub fn new(height: u32, char_width: u32, char_height: u32) -> Terminal {
		Terminal{commandline: String::new(), commands: Vec::new(), outputs: Vec::new(), height, char_width, char_height, on: false, h: 0}
	}
	pub fn toggle(&mut self) {
		self.on = !self.on;
	}
	pub fn is_active(&self) -> bool {
		self.on
	}
	pub fn update_commandline(&mut self, commandline: String) {
		self.commandline = commandline;
	}
	pub fn append_commandline(&mut self, commandline: &str) {
		self.commandline += commandline;
	}
	pub fn backspace(&mut self) {
		if self.commandline.len() > 0 {
			self.commandline = String::from(&self.commandline[0..self.commandline.len()-1]);
		}
	}
	pub fn process_commandline(&mut self, ctx: rlua::Context) {
		self.process_command(ctx, self.commandline.clone());
	}
	pub fn process_command(&mut self, ctx: rlua::Context, command: String) {
		self.commandline = String::new();
		self.commands.insert(0, command);
		let output = ctx.load(&self.commands[0]).eval::<rlua::Value>();
		if let Err(e) = output {
			self.outputs.insert(0, e.to_string());
		} else if let Ok(v) = output {
			match v {
				rlua::Value::Nil => self.outputs.insert(0, "Nil".to_string()),
				rlua::Value::Boolean(v) => self.outputs.insert(0, format!("{}", v)),
				rlua::Value::Integer(v) => self.outputs.insert(0, format!("{}", v)),
				rlua::Value::Number(v) => self.outputs.insert(0, format!("{}", v)),
				rlua::Value::String(v) => self.outputs.insert(0, format!("{}", v.to_str().unwrap())),
				v => self.outputs.insert(0, format!("{:?}", v)),
				
			}
		}
		println!("output: {}", self.outputs[0]);
	}
	pub fn draw_string(&self, canvas: &mut Canvas<Window>, font: &Texture, string: &str, line: u32) {
		canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
		for (i, c) in string.chars().enumerate() {
			let position = c as i32 - 32;
			let charx = (position % (font.query().width / self.char_width) as i32) * self.char_width as i32;
			let chary = (position / (font.query().width / self.char_width) as i32) * self.char_height as i32;
			let _ = canvas.copy(font, 
				Rect::new(charx, chary, self.char_width, self.char_height),//Rect::new((position * self.char_width as i32) % font.query().width as i32, font.query().height as i32 / position, self.char_width, self.char_height),
				Rect::new(i as i32 * self.char_width as i32, line as i32 * self.char_height as i32, self.char_width, self.char_height)).unwrap();
		}

	}
}

use crate::sdl_renderer::{Render, SdlRenderer};
impl Render for Terminal {
	fn render(&mut self, r: &mut SdlRenderer) {
		let font = &r.textures["font-oldschool"];
		if !self.on && self.h > 0 {
			self.h -= 20;
		}
		if self.on && self.h < self.height {
			self.h += 20;
		}
		//draw terminal
		r.canvas.set_draw_color(Color::RGBA(0, 0, 0, 127));
		let _ = r.canvas.fill_rect(Rect::new(0, self.height as i32-self.h as i32, r.screen_width, self.h));
		if self.h >= self.height {
			//draw text
			let line_height = self.height / self.char_height - 3;
			let _ = r.canvas.fill_rect(Rect::new(0, (line_height as i32 + 2) * self.char_height as i32, r.screen_width, self.char_height));
			self.draw_string(&mut r.canvas, font, &self.commandline, line_height + 2);
			for (line, command) in self.commands.iter().enumerate() {
				if (line as u32 + 3) * self.char_height as u32 > self.height / 2 {
					break;
				}
				self.draw_string(&mut r.canvas, font, &command, line_height - (line as u32 * 2 + 1));
				self.draw_string(&mut r.canvas, font, &self.outputs[line], line_height - (line as u32 * 2));
			}

	    	self.draw_string(&mut r.canvas, font, "lua dev console 0.2.8", 0);
	    }

	}
}