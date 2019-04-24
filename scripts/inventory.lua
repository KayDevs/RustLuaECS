InventorySystem = {
	inventories = {}
}

function InventorySystem:spawn(o)
	self.inventories[o.id] = {show = false, items = {}}
end
function InventorySystem:tick() 
	for k,v in pairs(self.inventories) do
		if v.show then
			print("entity #"..k.."'s inventory: ")
			tprint(v.items, 0)
		end
	end
end
function InventorySystem:get(id) 
	return self.inventories[id]
end

function InventorySystem:is_show(id) 
	return self.inventories[id].show
end
function InventorySystem:show(id, show)
	self.inventories[id].show = show
end

world:add_system(InventorySystem, "InventorySystem", "Inventory")