# Prototypes

These are some prototypes of games made with Asterism.

- `paddles`: remake of Atari Pong. Outputs to a framebuffer using the Pixels crate, as well as the terminal. Logics used: control, physics, collision, resource.
- `air_hockey`: Paddles, but flipped 180 degrees and with both horizontal and vertical movement. Logics used: control, physics, collision, resource.
- `paddles-macroquad`: Same as above, but with the crate `macroquad` for rendering to move away from framebuffer graphics.
- `jumper`: simple 2d platformer. Outputs to a framebuffer using the Pixels crate. Logics used: control, physics, collision, entity-state.
- `maze-minigame`: top-down navigation game. Outputs to a framebuffer using the Pixels crate. Has some destroyable items and portals (from Portal 2). Logics used: collision, resource, linking.
    - Collision and physics aren't currently implemented with Asterism.
- `yarn` (or, _sick day blues_): text game about being sick (oversimplified port of an old Twine game made by Cynthia). Outputs to the terminal. Logics used: linking, resource.
- `quest`, `archer_game`: Games made with WGPU graphics. These don't use Asterism.

<!-- most of the text here is originally from a writeup for a demo from Summer 2020, by Cynthia Li and Julie Ye: https://pom-itb-gitlab01.campus.pomona.edu/cxla2018/asterism-demo. -->
