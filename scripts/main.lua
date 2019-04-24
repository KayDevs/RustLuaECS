InfoSystem = {
	names = {}
}
function InfoSystem:spawn(o)
	self.names[o.id] = o.name
end
function InfoSystem:get(id) 
	return {name = self.names[id]}
end
function InfoSystem:set(id, o)
	self.names[id] = o.name
end
world:add_system(InfoSystem, "InfoSystem", "Info")

player = {
	Render = {
		sprite = "player_main",
		animations = {
			idle = {
				frame_width = 32,
				speed = 0,
				current_frame = 2,
			},
		},
		animation = "idle",
		z_index = 1,
	},
	---[[
	Physics = {
		position = {x = 32, y = 32},
		solid = true,
		dynamic = false,
		velocity = {x = 2, y = 2},
		acceleration = {x = 0, y = 0},
		angle = 45.0
	},
	--]]
	Collision = {
		respond_solid = true,
		--solid type?
		callback = function(self, other)
			print("fucka you", world.get("Info", other).name)
			world.system_update({"PhysicsComponent"}, function(p) p.velocity.x = -1 end)
		end
	},
	Input = {
		callback = function(self, input)
			world.entity_update(self, {"Position"}, function(pos) 
				if input.keyboard.Left then
					pos.x = pos.x - 1
				end
				if input.keyboard.Right then
					pos.x = pos.x + 1
				end
			end)
			if input.keyboard.A then
				print("A????")
			end
			if input.keybaord.I then
				InventorySystem:show(self, ~InventorySystem:is_show(self))
			end
		end
	},
	Inventory = {},
	Info = {
		name = "Kay"
	}
}

PlayerGlowSystem = {
	counter = 0
}
function PlayerGlowSystem:tick() 
	self.counter = self.counter + 1 / (2 * math.pi);
	--print(1 + math.sin(self.counter))
	world:entity_update(player_id, {'Render'}, function(r)
		offset = math.floor(16 * (1 + math.sin(self.counter / 2)))
		r.width = 32 + offset
		r.height = 32 + offset
		return r
	end)
end
world:add_system(PlayerGlowSystem, "PlayerGlowSystem", "PlayerGlowSystem")

local ReadOnly = {
}

local function check(tab, name, value)
  if rawget(ReadOnly, name) then
    error(name ..' is a read only variable', 2)
  end
  rawset(tab, name, value)
end

setmetatable(_G, {__index=ReadOnly, __newindex=check})

ReadOnly.player_id = world:spawn(player)
print("player id:", player_id)
tprint(world:get("Info", player_id))

--identity theft is serious
clone_id = world:spawn{Info = world:get("Info", player_id)}
tprint(world:get("Info", clone_id))
--make him a real boy
world:entity_update(clone_id, {"Info"}, function(i) i.name = "his own thang!!" return i end)
tprint(world:get("Info", clone_id))

PlayerFollowerSystem = {
	ids = {player_id},
	smoothing = 5,
	currently_lit = 1,
}
for i = 1,9 do
	table.insert(PlayerFollowerSystem.ids, world:spawn({Render={sprite="player_main", animations={idle={frame_width=32}}, animation="idle", z_index=-i, x = 0, y = 0}}))
end
function PlayerFollowerSystem:tick()
	for i = 1,10 do
		world:entity_update(self.ids[i], {'Render'}, function(r) r.animations["idle"].current_frame = 1 return r end)
	end
	self.currently_lit = self.currently_lit + 15/60 --1.5 revolutions per second
	if math.floor(self.currently_lit) > 10 then
		self.currently_lit = 1
	end
	world:entity_update(self.ids[math.floor(self.currently_lit)], {'Render'}, function(r) r.animations["idle"].current_frame = 2 return r end)

	for f = 2,10 do
		world:entity_update(self.ids[f], {"Render"}, function(p) 
			local p2 = world:get("Render", self.ids[f - 1])
			local distance = math.sqrt((p2.x - p.x)^2 + (p2.y - p.y)^2)
			local direction = math.atan(p2.y - p.y, p2.x - p.x)
			p.x = p.x + math.cos(direction) * distance / self.smoothing
			p.y = p.y + math.sin(direction) * distance / self.smoothing
		return p end)
	end
end
world:add_system(PlayerFollowerSystem, "PlayerFollowerSystem", "PlayerFollowerSystem")

PlayerBoundsSystem = {}
function PlayerBoundsSystem:tick()
	world:entity_update(player_id, {"Physics"}, function(p) 
		--[[
		if p.position.x + 16 > 640 then 
			p.position.x = 640 - 16
			p.velocity.x = -math.random(4)
		end
		if p.position.x - 16 < 0 then
			p.position.x = 0 + 16
			p.velocity.x = math.random(4)
		end
		if p.position.y + 16 > 400 then 
			p.position.y = 400 - 16
			p.velocity.y = -math.random(4)
		end
		if p.position.y - 16 < 0 then
			p.position.y = 0 + 16
			p.velocity.y = math.random(4)
		end
		--]]
		if p.position.x + 16 > 640 or p.position.x - 16 < 0 then
			p.velocity.x = p.velocity.x * -1
		end
		if p.position.y + 16 > 400 or p.position.y - 16 < 0 then
			p.velocity.y = p.velocity.y * -1
		end
		return p
	end)
end
world:add_system(PlayerBoundsSystem, "PlayerBoundsSystem", "PlayerBoundsSystem")

PlayerMouseSystem = {}
function PlayerMouseSystem:tick()
	world:entity_update(player_id, {"Physics"}, function(p)
		p.position.x = mouse_x
		p.position.y = mouse_y
		return p
	end)
end
--world:add_system(PlayerMouseSystem, "PlayerMouseSystem", "PlayerMouseSystem")

world:system_update({"Info"}, function(i)
	world:entity_update(player_id, {"Info"}, function(j) j.name = "???" return j end)
	i = i .. i
	return i
end)
tprint(world:get("Info", player_id))

EnemySystem = {
	enemies = {}
}
function EnemySystem:spawn(o)
	self.enemies[o.id] = {}
end
function EnemySystem:get(id)
	return self.enemies[id]
end
function EnemySystem:set(id, val)
	self.enemies[id] = val
end
world:add_system(EnemySystem, "EnemySystem", "Enemy")

enemy = {
	Enemy = {},
}
enemy_id = world:spawn(enemy)
print(world:get("Enemy", enemy_id) ~= nil)

world:entity_update(enemy_id, {'Enemy'}, function(e) return nil end)

world:system_update({"Enemy"}, function()
	print("Garble Garble")
	player_pos = world:get("Physics", player_id).position
	print("I am angery!! uuuuhhh I see player at", string.format("(%d, %d)", player_pos.x, player_pos.y))
end)

print("still, ", world:get("Enemy", enemy_id) ~= nil)

--world:delete_entity(player_id)
