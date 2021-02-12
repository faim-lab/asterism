# notes/TODOs

- i wonder if we should have stronger opinions about how (un)projection happens, since it can be easy to miss changes. having better functions to move between the game state and logics would help, i think
- how expensive is stuff like `get_action`/`get_pos_for_entity`? since worst case it has to iterate through the entire array
    - iterators for unprojection?
- searching through `last_frame_keys` is probably expensive as well

collision/physics:

- change restitution so that each contact is only in the contacts vec once
- we keep track of three kinds of entities:
    + non-solid: penetrable, can't be restituted
    + solid and fixed: non-penetrable, can't be restituted
    + solid and unfixed: non-penetrable, can be restituted

    and i wonder if we should differentiate between them somehow
- look into rapier
- combine physics and collision...? considering how `apple-catching` deals with bouncing apples currently, this makes sense
- `get_position_for_entity` except for physics positions/velocities/accelerations?
- should velocity be stored in the game state or in the physics logic
- have a field for `collided_last_frame`...? similar to `last_frame_keys`, or should that be in the game state

entity state:

- basically make this logic more ergonomic to use because all these usizes SUCK
- some method that gets the id of the state given the edge?
- some kind of entity id...?
- states field should give you... a StateID... not a usize
- fn that can change values in the condition table during projecting
- this also applies to linking logics since they're still pretty much the same thing

linking:

- type alias for NodeMap maybe, since it literally just holds a vec of nodes? `pub type NodeMap = Vec<Nodes>;`
- some sort of graph trait......?

control:

- unprojection w/ `get_action_in_set` is _very_ verbose
- consider making control mappings reconstruct themselves every frame...?
- controller support?
    - i think this would help with thinking about analog vs digital inputs because keyboards... just don't have analog input, and mouse input is something entirely different altogether.

# DONE
- [x] looking at `control.get_action_in_set` and `collision.get_position_for_entity`, try to avoid indexing by .0/.1 when getting values. it feels unintuitive and i always end up looking up the docs every time
    - made `get_action`/`get_action_in_set` return `Values` since that's already a structure we have in the logic anyway
    - there's no reason `collision.get_position_for_entity` should return the half size
- [x] bump up the `macroquad`/`glam` version since they made `Vec2`s nicer
    - set the dependency to a git hash instead of a crates.io version
- [x] use `where` keyword for types
- [x] only add each contact once to the contacts struct? make the nested loop when checking intersections `self.positions[i..].iter()` &c
- [x] consider some way of setting the min and max values of a resource
