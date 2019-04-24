use serde::{Serialize, Deserialize};
use crate::world::{World, NativeSystem};
use std::collections::HashMap;

//use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderInfo {
	pub sprite: String,
	#[serde(default)]
	pub animation: String,
	#[serde(default)]
	pub animations: HashMap<String, Animation>,
	#[serde(default)]
	pub x: f64,
	#[serde(default)]
	pub y: f64,
	#[serde(default)]
	pub width: u32,
	#[serde(default)]
	pub height: u32,
	#[serde(default)]
	pub rotation: f64,
	#[serde(default)]
	pub z_index: i32,
}


fn one() -> u32 {
	1
}
fn fone() -> f64 {
	1.0
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Animation {
	pub frame_width: u32, //must be specified
	#[serde(default)] //if frame_height = 0 then frame_height = texture_height
	pub frame_height: u32,
	#[serde(default)]
	pub row: u32,
	#[serde(default = "one")] //1 indexed
	pub first: u32,
	#[serde(default)] //if end = 0 then end = texture_width / frame_width
	pub last: u32,
	#[serde(default)]
	pub speed: f64,
	#[serde(default = "fone")]
	pub current_frame: f64, //it's actually a float here but gets rounded on get/set
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
	pub x: f64,
	pub y: f64,
	pub width: u32,
	pub height: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnimationComponent {
	pub animations: HashMap<String, Animation>,
	pub animation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderSystem {
	//hashmap for quickest lookup
	sprites: HashMap<usize, String>,
	frames: HashMap<usize, Frame>,
	ordering: HashMap<usize, i32>,
	rotations: HashMap<usize, f64>,
	//animations require indirect lookup, but faster iteration via Vec
	entities: HashMap<usize, usize>,
	animations: Vec<AnimationComponent>,

	pub camera_x: i32,
	pub camera_y: i32,
}

impl RenderSystem {
	pub fn set_position(&mut self, entity: usize, x: f64, y: f64) {
		if let Some(f) = self.frames.get_mut(&entity) {
			f.x = x;
			f.y = y;	
		}
	}
	pub fn set_rotation(&mut self, entity: usize, rot: f64) {
		if let Some(angle) = self.rotations.get_mut(&entity) {
			*angle = rot;
		}
	}
}

impl NativeSystem for RenderSystem {
	fn new() -> RenderSystem {
	    RenderSystem{camera_x: 0, camera_y: 0, sprites: HashMap::new(), entities: HashMap::new(), animations: Vec::new(), frames: HashMap::new(), ordering: HashMap::new(), rotations: HashMap::new()}
	}
	fn spawn(&mut self, entity: usize, object: rlua::Value) {
		if let Ok(RenderInfo{sprite, animations, animation, x, y, width, height, z_index, rotation}) = rlua_serde::from_value(object) {
			self.sprites.insert(entity, sprite);
			self.frames.insert(entity, Frame{x, y, width, height});
			self.ordering.insert(entity, z_index);
			self.entities.insert(entity, self.entities.len());
			self.animations.push(AnimationComponent{animations, animation});
			self.rotations.insert(entity, rotation);
        }
	}
	fn tick(&mut self, _: &World) {
		for a in &mut self.animations {
			//this is needed bc they might not have animations at all
			if let Some(a) = a.animations.get_mut(&a.animation) {
				if a.last != 0 {
					a.current_frame += a.speed;
					//rendered frame index = floor(current_frame)
					//if speed is 0.1
					//1.0-1.9 = frame 1; 2.0-2.9 = frame 2; etc.
					//resets when no longer currently on last frame i.e. >= last + 1
					//i.e. if last frame is 3 then reset > 3.9, or, >= 4.0
					if a.current_frame >= (a.last + 1) as f64 {
						a.current_frame = a.first as f64;
					}
				}
			}
		}
	}
	fn get<'lua>(&self, ctx: rlua::Context<'lua>, entity: usize) -> rlua::Value<'lua> {
		if let Some(sprite) = self.sprites.get(&entity) {
			let Frame{x, y, width, height} = self.frames[&entity];
			let z_index = self.ordering[&entity];
			let rotation = self.rotations[&entity];
			let i = self.entities[&entity];
			let animation_component = &self.animations[i];
			rlua_serde::to_value(ctx, RenderInfo{
				sprite: sprite.to_string(), 
				animations: animation_component.animations.clone(), 
				animation: animation_component.animation.clone(),
				x, y,
				width,
				height,
				z_index,
				rotation}).unwrap()
		} else {
			rlua::Value::Nil
		}
	}
	fn set<'lua>(&mut self, entity: usize, value: rlua::Value<'lua>) {
		if self.sprites.contains_key(&entity) {
			if let Ok(RenderInfo{sprite, animation, animations, x, y, width, height, z_index, rotation}) = rlua_serde::from_value(value) {
				self.sprites.insert(entity, sprite);
				self.frames.insert(entity, Frame{x, y, width, height});
				self.ordering.insert(entity, z_index);
				self.rotations.insert(entity, rotation);
				self.animations[self.entities[&entity]] = AnimationComponent{animations, animation};
			}
		}
	}
	fn save(&self) -> serde_json::Value {
		serde_json::to_value(self).unwrap()
	}
}


use crate::sdl_renderer::{SdlRenderer, Render};
impl Render for RenderSystem {
	fn render(&mut self, r: &mut SdlRenderer) {
		let mut render_queue = Vec::new();
		for (i, z) in &self.ordering {
			render_queue.push((z, i));
		}
		render_queue.sort_unstable();
		for (_, i) in render_queue {

			let tex = &r.textures[&self.sprites[i]];
			let frame = &mut self.frames.get_mut(i).unwrap();
			let mut src_rect = None;
			if let Some(&i) = self.entities.get(i) {
				let current_animation = self.animations[i].animation.clone();
				let mut animation = self.animations.get_mut(i).unwrap().animations.get_mut(&current_animation).unwrap();
				if animation.last == 0 {
					//if animation end is not defined (=0), set it to the last frame in the row
					animation.last = tex.query().width / animation.frame_width;
				}
				if animation.frame_height == 0 {
					animation.frame_height = tex.query().height;
				}
				if animation.frame_width == 0 {
					animation.frame_width = tex.query().width;
				}

				if frame.width == 0 {
					frame.width = animation.frame_width;
				}
				if frame.height == 0 {
					frame.height = animation.frame_height;
				}
				//in order of priority:
				//width > frame_width > texture.width
				//height > frame_height > texture.height

				src_rect = Some(Rect::new(
					((animation.current_frame - 1.0).floor() * animation.frame_width as f64) as i32, 
					animation.row as i32,
					animation.frame_width, 
					animation.frame_height));
			}

			//TODO: maybe let the user select between these two with a 'centered' boolean
			let draw_rect = Rect::new(frame.x as i32 - frame.width as i32 / 2 - self.camera_x, frame.y as i32 - frame.height as i32 / 2 - self.camera_y, frame.width, frame.height);
			//let draw_rect = Rect::new(frame.x as i32, frame.y as i32, frame.width, frame.height);

			if draw_rect.x + draw_rect.w > 0 
			&& draw_rect.x < r.screen_width as i32
			&& draw_rect.y + draw_rect.h > 0
			&& draw_rect.y < r.screen_height as i32 {
				let _ = r.canvas.copy_ex(tex, src_rect, draw_rect, self.rotations[i], None, false, false);
			}
		}
	}
}
