# Notes

## Data Driven Stuff
- mappings are defined through types
- query/condition tables?
    - There's a closed set of possible query tables based on logics (predicates \cup syntheses?), which you would then compose into more complex conditions
- Is it possible to generate an engine through macros?

## Logics Stuff

### Collision/Physics:
- three kinds of entities:
    + non-solid: penetrable, can't be restituted
    + solid and fixed: non-penetrable, can't be restituted
    + solid and unfixed: non-penetrable, can be restituted
- look into `rapier`

### Entity-state:
- Basically make this logic more ergonomic to use because all these usizes suck
- Some method that gets the id of the state given the edge?
- Some kind of entity ID...?
- `states` field should give you... a StateID... not a usize
- This also applies to linking logics since they're still pretty much the same thing

### Linking:
- Type alias for NodeMap maybe, since it literally just holds a vec of nodes? `pub type NodeMap = Vec<Nodes>;`
- Some sort of graph trait......?
    - State machine struct used for both the game state at large and also this?

### Control:
- Unprojection w/ `get_action_in_set` is _very_ verbose
- Consider making control mappings reconstruct themselves every frame...?
- I think controller supoprt would help with thinking about analog vs digital inputs because keyboards... just don't have analog input (mouse input = control + selection). But also, whatever

# Old stuff
- looking at `control.get_action_in_set` and `collision.get_position_for_entity`, try to avoid indexing by .0/.1 when getting values.
    - made `get_action`/`get_action_in_set` return `Values` since that's already a structure we have in the logic anyway
    - there's no reason `collision.get_position_for_entity` should return the half size
- bump up the `macroquad`/`glam` version since they made `Vec2`s nicer
- use `where` keyword for types
- only add each contact once to the contacts struct? make the nested loop when checking intersections `self.positions[i..].iter()` &c
- consider some way of setting the min and max values of a resource
- change restitution so that each contact is only in the contacts vec once

- ~~combine physics and collision...?~~
- ~~have a field for `collided_last_frame`...? similar to `last_frame_keys`, or should that be in the game state~~
- ~~i wonder if we should have stronger opinions about how (un)projection happens, since it can be easy to miss changes. having better functions to move between the game state and logics would help, i think~~
