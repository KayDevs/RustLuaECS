extern crate serde;
extern crate serde_json;
extern crate rlua;
extern crate rlua_serde;
#[macro_use]
extern crate rust_embed;

extern crate sdl2;
use std::error::Error;
use std::path::Path;

mod world;
use world::NativeSystem;
mod physics;
mod render;
mod sdl_renderer;
use sdl_renderer::Render;
mod terminal;
use terminal::Terminal;

#[derive(RustEmbed)]
#[folder="resources/"]
struct Resources;

#[derive(RustEmbed)]
#[folder="scripts/"]
struct Scripts;


use std::sync::Arc;
fn main() ->  Result<(), Box<Error>> {

	let sdl_context = sdl2::init()?;
	let mut sdl_renderer = sdl_renderer::SdlRenderer::new(&sdl_context, "luasys", 640, 400)?;

	for r in Resources::iter() {
		//I hate the Path/OsStr APIs
		let string = r.to_string();
		let path = Path::new(&string);
		let file_stem = path.file_stem().unwrap().to_str().unwrap();
		sdl_renderer.insert_texture(file_stem.to_string(), &Resources::get(&r).unwrap()); 
	}

	let mut term = Terminal::new(sdl_renderer.screen_height, 7, 9);

	let lua = rlua::Lua::new();
	let world = world::World::new();

	//start out with NativeSystems
	lua.context(|ctx| {
		world.add_native_system(ctx, Box::new(physics::PhysicsSystem::new()), "PhysicsSystem", "Physics");
		world.add_native_system(ctx, Box::new(render::RenderSystem::new()), "RenderSystem", "Render");
	});

	//some nice functions
	lua.context(|ctx| {
		ctx.load(r#"
			function tprint (tbl, indent)
			if not indent then indent = 0 end
			for k, v in pairs(tbl) do
			formatting = string.rep("  ", indent) .. k .. ": "
			if type(v) == "table" then
			print(formatting)
			tprint(v, indent+1)
			else
			print(formatting .. tostring(v))
			end
			end
			end"#).exec().unwrap();
		ctx.load(r#"
			--prefer math.floor() but here's this just in case
			math.round = function(n) 
				return n >= 0.0 and n-n%-1 or n-n% 1  -- rounds away from zero, towards both infinities.
			end
		"#).exec().unwrap();
		ctx.load("math.randomseed(os.time())").exec().unwrap();
	});

	//give the world to lua
	let w = world::WorldRef(Arc::new(world));
	lua.context(|ctx| {
		ctx.globals().set("world", w.clone()).unwrap();
		//load scripts in scripts/ directory
		for s in Scripts::iter() {
			println!("loading {:?}", s);
			ctx.load(&Scripts::get(&s).unwrap()).exec().unwrap();
		}
	});


	//TODO: make a module for input handling, similar to sdl_renderer
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        //parse events
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        use sdl2::mouse::MouseButton;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                },
                Event::KeyDown{keycode: Some(keycode), ..} => {
                	match keycode {
                		Keycode::Left => w.write_native_system("Render", |r: &mut render::RenderSystem| {r.camera_x -= 16}),
                		Keycode::Right => w.write_native_system("Render", |r: &mut render::RenderSystem| {r.camera_x += 16}),
                		Keycode::Up => w.write_native_system("Render", |r: &mut render::RenderSystem| {r.camera_y -= 16}),
                		Keycode::Down => w.write_native_system("Render", |r: &mut render::RenderSystem| {r.camera_y += 16}),
                		Keycode::Backquote => {
		                	if term.is_active() {
		                		sdl_renderer.video.text_input().stop();
		                	} else {
		                		sdl_renderer.video.text_input().start();
		                	}
		                	term.toggle();
                		},
                		Keycode::Return => {
			            	if term.is_active() {
			            		sdl_renderer.video.text_input().stop();
			                	lua.context(|ctx|{
			                		term.process_commandline(ctx);
			                	});
			                	//don't deactivate term unless they press backquote again
			                	sdl_renderer.video.text_input().start();
			            	}
                		},
                		Keycode::Backspace => {
				        	if term.is_active() {
				        		term.backspace();
				        	}
                		},
                		_ => {}
                	}
                },
                //event::mouseclick: go through entities, see if mouse is between x,y and w,h
                Event::MouseButtonDown{mouse_btn: MouseButton::Left, x: mx, y: my, ..} => {
                	if term.is_active() {
                		lua.context(|ctx| {
							for i in 0..w.size() {
								if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
										ctx.load(&format!("world:get('Render', {}).x", i)).eval::<f64>(), 
										ctx.load(&format!("world:get('Render', {}).y", i)).eval::<f64>(), 
										ctx.load(&format!("world:get('Render', {}).width", i)).eval::<u32>(),
										ctx.load(&format!("world:get('Render', {}).height", i)).eval::<u32>()) {
									let (x, y, w, h) = (x as i32, y as i32, w as i32, h as i32);
									if mx > x && mx < x+w && my > y && my < y+h {
										println!("id: {}", i);
										term.append_commandline(&i.to_string());
										break;
									}
								}
							}
						});
                	}
                }

                Event::TextEditing{text, ..} => {
                	//println!("editing: {}", text);
                	term.update_commandline(text);
                }
                Event::TextInput{text, ..} => {
            		//println!("input: {}", text);
            		term.append_commandline(&text);
                }
                _ => {}
            }
        }

        lua.context(|ctx| {
			//LOGIC
			ctx.load("world:tick()").exec().unwrap();
            ctx.globals().set("mouse_x", event_pump.mouse_state().x()).unwrap();
            ctx.globals().set("mouse_y", event_pump.mouse_state().y()).unwrap();
        });


		//RENDER

		sdl_renderer.clear(200, 200, 255);

		w.write_native_system("Render", |r: &mut render::RenderSystem| {
			w.write_native_system("Physics", |ph: &mut physics::PhysicsSystem| {
				for (_, &e) in &ph.entities {
					r.set_position(e, ph.positions[e].x, ph.positions[e].y);
					r.set_rotation(e, ph.angles[e]);
				}
			});
			r.render(&mut sdl_renderer);
			//r.render(&mut r);
		});

		term.render(&mut sdl_renderer);

		sdl_renderer.present();
    }

	Ok(())
}
