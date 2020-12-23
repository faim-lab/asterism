# notes/TODOs

- i wonder if we should have stronger opinions about how (un)projection happens, since it can be easy to miss changes. having better functions to move between the game state and logics would help, i think
- looking at `control.get_action_in_set` and `collision.get_position_for_entity`, try to avoid indexing by .0/.1 when getting values. it feels unintuitive and i always end up looking up the docs every time
- consider bumping up the `macroquad`/`glam` version since they made `Vec2`s nicer ([you can access `x` and `y` as fields now](https://docs.rs/macroquad/0.3.0-alpha.14/macroquad/math/struct.Vec2.html#fields)), depending on how many things this breaks
    - cargo doesn't seem to respect that i've put in macroquad's version as `0.3.0-alpha.9` and downloads `0.3.0-alpha.14` anyway, so maybe set the dependency to a git hash instead

collision/physics:

- look into rapier
- combine physics and collision...? considering how `apple-catching` deals with bouncing apples currently, this makes sense

entity state:

- basically make this logic more ergonomic to use because all these usizes SUCK
- some method that gets the id of the state given the edge?
- some kind of entity id...?
- states field should give you... a StateID... not a usize
- fn that can change values in the condition table during projecting

resource:

- consider some way of setting the min and max values of a resource

linking:

- type alias for NodeMap maybe, since it literally just holds a vec of nodes? `pub type NodeMap = Vec<Nodes>;`
- some sort of graph trait......?

control:

- maybe make control mappings reconstruct themselves every frame...? i'm not sure
- consider using `where` keyword for types, for readability
- controller support?
    - i think this would help with thinking about analog vs digital inputs because keyboards... just don't have digital input, and mouse input is something entirely different altogether.
