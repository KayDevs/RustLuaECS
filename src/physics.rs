use crate::world::{NativeSystem, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PhysicsObject {
    #[serde(default)]
    pub position: Vector2,
    #[serde(default)]
    pub velocity: Vector2,
    #[serde(default)]
    pub acceleration: Vector2,
    #[serde(default)]
    pub angle: f64,
}

//exposed position + angle for rendering
#[derive(Debug, Serialize, Deserialize)]
pub struct PhysicsSystem {
    pub entities: HashMap<usize, usize>,
    pub positions: Vec<Vector2>,
    velocities: Vec<Vector2>,
    accelerations: Vec<Vector2>,
    pub angles: Vec<f64>,
}

impl NativeSystem for PhysicsSystem {
    fn new() -> PhysicsSystem {
        PhysicsSystem {
            entities: HashMap::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            angles: Vec::new(),
        }
    }
    fn globals(&self, ctx: rlua::Context) {
        ctx.load(
            r#"
			function set_position(id, x, y) 
				world:entity_update(id, {'Physics'}, function(p) 
					p.position.x = x 
					p.position.y = y
					return p 
				end) 
			end"#,
        )
        .exec()
        .unwrap();
        ctx.load(
            r#"
			function set_velocity(id, x, y)
				world:entity_update(id, {'Physics'}, function(p)
					p.velocity.x = x
					p.velocity.y = y
					return p
				end)
			end"#,
        )
        .exec()
        .unwrap();
    }
    fn tick(&mut self, _: &World) {
        //println!("{:?}", self);
        for i in 0..self.entities.len() {
            //self.velocities[i].y += 9.81 / 96.0; //32 pixels ~= 1 foot; 96 pixels = 1 meter
            self.velocities[i].x += self.accelerations[i].x;
            self.velocities[i].y += self.accelerations[i].y;
            self.positions[i].x += self.velocities[i].x;
            self.positions[i].y += self.velocities[i].y;
            //self.angles[i] += 2.0 / self.velocities[i].x; //really basic visual effect
        }
    }
    fn save(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn spawn(&mut self, entity: usize, object: rlua::Value) {
        if let Ok(PhysicsObject {
            position,
            velocity,
            acceleration,
            angle,
        }) = rlua_serde::from_value(object)
        {
            self.entities.insert(entity, self.positions.len());
            self.positions.push(position);
            self.velocities.push(velocity);
            self.accelerations.push(acceleration);
            self.angles.push(angle);
        } else {
            panic!("Could not parse object")
        }
    }
    fn get<'lua>(&self, ctx: rlua::Context<'lua>, entity: usize) -> rlua::Value<'lua> {
        if let Some(&i) = self.entities.get(&entity) {
            rlua_serde::to_value(
                ctx,
                PhysicsObject {
                    position: self.positions[i],
                    velocity: self.velocities[i],
                    acceleration: self.accelerations[i],
                    angle: self.angles[i],
                },
            )
            .unwrap()
        } else {
            rlua::Value::Nil
        }
    }
    fn set<'lua>(&mut self, entity: usize, value: rlua::Value<'lua>) {
        if let Some(&i) = self.entities.get(&entity) {
            if let Ok(PhysicsObject {
                position,
                velocity,
                acceleration,
                angle,
            }) = rlua_serde::from_value(value)
            {
                self.positions[i] = position;
                self.velocities[i] = velocity;
                self.accelerations[i] = acceleration;
                self.angles[i] = angle;
            }
        }
    }
}
