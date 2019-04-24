use std::collections::HashMap;
use std::fmt::Debug;

pub trait NativeSystem: Debug + Send + Sync + 'static {
	fn new() -> Self where Self: Sized;
	fn tick(&mut self, world: &World);
	fn globals(&self, _ctx: rlua::Context) {}
	fn spawn(&mut self, entity: usize, object: rlua::Value);
	fn get<'lua>(&self, ctx: rlua::Context<'lua>, entity: usize) -> rlua::Value<'lua>;
	fn set<'lua>(&mut self, entity: usize, value: rlua::Value<'lua>);
	fn save(&self) -> serde_json::Value;
}

//The folowing set of functions allow for getting a specific NativeSystem from the world, and will not work otherwise
pub trait AnyNativeSystem: NativeSystem + std::any::Any {
    fn as_any(&self) -> &std::any::Any;
    fn as_any_mut(&mut self) -> &mut std::any::Any;
}


impl dyn AnyNativeSystem {
    fn to_system<T: NativeSystem>(&self) -> Option<&T> {
        std::any::Any::downcast_ref::<T>(self.as_any())
    }
    fn to_system_mut<T: NativeSystem>(&mut self) -> Option<&mut T> {
    	std::any::Any::downcast_mut::<T>(self.as_any_mut())
    }
}

impl<T> AnyNativeSystem for T
    where T: NativeSystem + std::any::Any
{
    fn as_any(&self) -> &std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut std::any::Any {
        self
    }
}

enum System {
	NativeSys(Box<dyn AnyNativeSystem>),
	LuaSys(rlua::RegistryKey),
}

impl System {
	fn as_native_system(&self) -> &Box<dyn AnyNativeSystem> {
		if let System::NativeSys(s) = self {
			s
		} else {
			panic!("not a NativeSystem");
		}
	}
	fn as_native_system_mut(&mut self) -> &mut Box<dyn AnyNativeSystem> {
		if let System::NativeSys(s) = self {
			s
		} else {
			panic!("not a NativeSystem");
		}
	}
}

use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};

//Component Name, System
pub struct World {
	systems: RwLock<HashMap<String, RwLock<System>>>,
	base_id: AtomicUsize,
}

#[allow(unused)]
impl World {
	pub fn new() -> World {
		World {
			base_id: AtomicUsize::new(0),
			systems: RwLock::new(HashMap::new()),
		}
	}
	pub fn tick(&self, ctx: rlua::Context) {
		for (_, v) in self.systems.read().unwrap().iter() {
			match *v.write().unwrap() {
				System::LuaSys(ref mut v) => {
					let table: rlua::Table = ctx.registry_value(&v).unwrap();
					if let Ok(function) = table.get::<_, rlua::Function>("tick") {
						function.call::<rlua::Table, ()>(table).unwrap();
					}
				},
				System::NativeSys(ref mut v) => {
					v.tick(self);
				}
			}
		}
	}

	pub fn spawn<'lua>(&self, ctx: rlua::Context<'lua>, components: rlua::Table<'lua>) -> usize {
		let id = self.base_id.fetch_add(1, Ordering::SeqCst);
		for (k, v) in self.systems.read().unwrap().iter() {
			if let Ok(object) = components.get::<&str, rlua::Table>(k) {
				match *v.write().unwrap() {
					System::NativeSys(ref mut v) => v.spawn(id, rlua::Value::Table(object)),
					System::LuaSys(ref mut v) => {
						object.set("id", id).unwrap();
						let table: rlua::Table = ctx.registry_value(&v).unwrap();
						if let Ok(function) = table.get::<_, rlua::Function>("spawn") {
							function.call::<(rlua::Table, rlua::Table), ()>((table, object)).unwrap();
						}
					}
				}
			}		
		}
		id
	}

	pub fn get<'lua>(&self, ctx: rlua::Context<'lua>, name: String, entity: usize) -> rlua::Value<'lua> {
		if let Some(ref sys_lock) = self.systems.read().unwrap().get(&name) {
			match *sys_lock.read().unwrap() {
				System::NativeSys(ref sys) => sys.get(ctx, entity).clone(),
				System::LuaSys(ref sys) => {
					let table: rlua::Table = ctx.registry_value(&sys).unwrap();
					if let Ok(function) = table.get::<_, rlua::Function>("get") {
						function.call::<(rlua::Table, usize), rlua::Value>((table, entity)).unwrap()
					} else {
						rlua::Value::Nil
					}
				}
			}
		} else {
			rlua::Value::Nil
		}
	}
	pub fn set<'lua>(&self, ctx: rlua::Context<'lua>, name: String, entity: usize, value: rlua::Value<'lua>) {
		if let Some(ref sys_lock) = self.systems.read().unwrap().get(&name) {
			match *sys_lock.write().unwrap() {
				System::NativeSys(ref mut sys) => sys.set(entity, value),
				System::LuaSys(ref sys) => {
					let table: rlua::Table = ctx.registry_value(&sys).unwrap();
					if let Ok(function) = table.get::<_, rlua::Function>("set") {
						function.call::<(rlua::Table, usize, rlua::Value), ()>((table, entity, value)).unwrap();
					}
				}
			}
		}
	}

	pub fn system_update<'lua>(&self, ctx: rlua::Context<'lua>, components: rlua::Table<'lua>, setter: rlua::Function<'lua>) {
		//idea: go through components, copy into closure, get them back out, put them back into systems
		'entities: for entity in 0..self.base_id.load(Ordering::SeqCst) {
			let mut entity_components = rlua::Variadic::new();

			//make sure entity has every component, put them into their own vec
			//clone it because we want to reuse it for the next entity
			for pair in components.clone().pairs::<usize, String>() {
				if let Ok((_, name)) = pair {
					match self.get(ctx, name, entity) {
						rlua::Value::Table(component) => {
							entity_components.push(component);
						},
						//if entity doesn't have any one of the components, skip it
						_ => continue 'entities
					}
				}
			}

			//call the function here (accepts a variadic, returns variadic)
			let returns = setter.call::<rlua::Variadic<rlua::Table>, rlua::Value>(entity_components.clone());

			//now put them back, consuming the variadic in the process
			for (i, table) in returns.into_iter().enumerate() {
				self.set(ctx, components.get(i+1).unwrap(), entity, table);
			}
		}
	}

	pub fn entity_update<'lua>(&self, ctx: rlua::Context<'lua>, entity: usize, components: rlua::Table<'lua>, setter: rlua::Function<'lua>) {
		//same idea as system_update, except instead of skipping entities, we just exit immediately.
		let mut entity_components = rlua::Variadic::new();

		//make sure entity has every component, put them into their own vec
		//clone it because we want to reuse it for the next entity
		for pair in components.clone().pairs::<usize, String>() {
			if let Ok((_, name)) = pair {
				match self.get(ctx, name, entity) {
					rlua::Value::Table(component) => {
						entity_components.push(component);
					},
					//if entity doesn't have any one of the components, skip it
					_ => return
				}
			}
		}

		//call the function here (accepts a variadic, returns variadic)
		let returns = setter.call::<rlua::Variadic<rlua::Table>, rlua::Value>(entity_components.clone());

		//now put them back, consuming the variadic in the process
		for (i, table) in returns.into_iter().enumerate() {
			self.set(ctx, components.get(i+1).unwrap(), entity, table);
		}
	}

	pub fn add_native_system(&self, ctx: rlua::Context, mut system: Box<AnyNativeSystem>, system_name: &str, object_name: &str) {
		system.globals(ctx);
		self.systems.write().unwrap().insert(object_name.to_string(), std::sync::RwLock::new(System::NativeSys(system)));
		//self.systems[&object_name.to_string()].write().unwrap().globals(ctx);
		//self.system_names.insert(object_name.to_string(), system_name.to_string())
	}

	pub fn read_native_system<Sys, F>(&self, object_name: &str, f: F)
	where Sys: NativeSystem, F: Fn(&Sys) {
		let systems_guard = self.systems.read().unwrap();
		let sysguard = systems_guard.get(object_name).unwrap().read().unwrap();
		if let Some(sys) = sysguard.as_native_system().to_system::<Sys>() {
			f(sys);
		}
	}
	pub fn write_native_system<Sys, F>(&self, object_name: &str, mut f: F)
	where Sys: NativeSystem, F: FnMut(&mut Sys) {
		let mut systems_guard = self.systems.read().unwrap();
		let mut sysguard = systems_guard.get(object_name).unwrap().write().unwrap();
		if let Some(sys) = sysguard.as_native_system_mut().to_system_mut::<Sys>() {
			f(sys);
		}
	}

	pub fn add_lua_system<'lua>(&self, ctx: rlua::Context<'lua>, system: rlua::Table<'lua>, system_name: String, object_name: String) {
		let regkey = ctx.create_registry_value(system).unwrap();
		self.systems.write().unwrap().insert(object_name, std::sync::RwLock::new(System::LuaSys(regkey)));
		//self.system_names.insert(object_name, system_name)
	}

	pub fn save(&self) -> serde_json::Value {
		//create json object where keys are SYSTEM_NAME and values are System::save()
		unimplemented!()
	}
}

use std::sync::Arc;
pub struct WorldRef(pub Arc<World>);
impl Clone for WorldRef {
	fn clone(&self) -> Self {
		WorldRef(self.0.clone())
	}
}

impl WorldRef {
	//delegates
	pub fn read_native_system<Sys, F>(&self, object_name: &str, f: F)
	where Sys: NativeSystem, F: Fn(&Sys) {
		self.0.read_native_system(object_name, f);
	}
	pub fn write_native_system<Sys, F>(&self, object_name: &str, f: F)
	where Sys: NativeSystem, F: FnMut(&mut Sys) {
		self.0.write_native_system(object_name, f);
	}
	pub fn size(&self) -> usize {
		self.0.base_id.load(Ordering::SeqCst)
	}
}

impl rlua::UserData for WorldRef {
	fn add_methods<'lua, M: rlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("spawn", |ctx, this, components: rlua::Table| {
			Ok(this.0.spawn(ctx, components))
		});
		methods.add_method("get", |ctx, this, (name, entity): (String, usize)| {
			Ok(this.0.get(ctx, name, entity))
		});
		methods.add_method("system_update", |ctx, this, (components, setter): (rlua::Table, rlua::Function)| {
			Ok(this.0.system_update(ctx, components, setter))
		});
		methods.add_method("entity_update", |ctx, this, (entity, components, setter): (usize, rlua::Table, rlua::Function)| {
			Ok(this.0.entity_update(ctx, entity, components, setter))
		});
		methods.add_method("add_system", |ctx, this, (system, system_name, object_name): (rlua::Table, String, String)| {
			Ok(this.0.add_lua_system(ctx, system, system_name, object_name))
		});
		methods.add_method("tick", |ctx, this, ()| {
			Ok(this.0.tick(ctx))
		});
		methods.add_method("size", |_, this, ()| {
			Ok(this.0.base_id.load(Ordering::SeqCst))
		});
	}
}
