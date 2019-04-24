# RustLuaECS
tried another approach to the whole ECS thing, this time the main focus being Systems.

In my [last project](https://github.com/KayDevs/homemade) the center focus of the code's layout was Components, and Systems in that engine were just simple functions/closures. I ran into many shortcomings - namely trying to access a bunch of data from the same struct in different locations in a thread-safe manner is hard. It's very tricky to work out game details when everything is under lock and key. Furthermore, this does NOT integrate with scripts well. I don't even know how to begin to think about how to expose a `RwLock<Box<AnyComponentVec>>` to Lua.
Finally, this design doesn't bode well for data efficiency; while I do have options for using a Vec/HashMap/BTreeMap, I would like each system to have finer control of how its memory is laid out.

So, this time I decided to make Systems the primary focus. Following the advice of a [bitsquid blog series](http://bitsquid.blogspot.com/2014/08/building-data-oriented-entity-system.html) I decided to give Systems utmost control over their components, and thus their memory layout, since they now own all their data. There is no centralized AnyMap of all the data in the game, it's decentralized and spread out to different systems that do different things. This also allows me to easily integrate Lua scripting - systems can now be either Native or Lua, the engine doesn't care.

The achilles heel of this engine, however, is this fact: communicating with Lua requires marshalling. Which is slow. It's also painfully redundant and time-consuming to write the same dozen variable names in the system's definition, in the serialized object definition, in getters, and in setters. This design got much further than my prior design, but it's still too time consuming for regular game development.

I'm uploading it here for (my) education's sake.

I might continue on it if I meet some personal requirements:
1. I ditch get/set in favor of having each system expose their own functions to Lua, at their discretion of course.
2. I have some mechanism to allow systems (Lua or native) to not have to keep track of the entities belonging to them. That should be managed somewhere else; it adds a lot of boilerplate when writing systems code.
