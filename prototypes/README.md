# Prototypes

These are some prototypes of games made with `asterism`.

- `yarn` (or, _sick day blues_): text game about being sick (oversimplified port of an old Twine game made by Cynthia). Outputs to the terminal. Logics used: linking, resource.
- `paddles`: remake of Atari Pong. Outputs to a framebuffer using the Pixels crate, as well as the terminal. Logics used: control, physics, collision, resource.
- `jumper`: simple 2d platformer. Outputs to a framebuffer using the Pixels crate. Logics used: control, physics, collision, entity-state.
- `maze-minigame`: top-down navigation game. Outputs to a framebuffer using the Pixels crate. Has some destroyable items and portals (from Portal 2). Logics used: collision, resource, linking.
    - Control and physics aren't currently implemented with Asterism.
- `quest`, `archer_game`: Games made with WGPU graphics. These don't use Asterism (yet?).

Paddles and maze-minigame (the original versions using the Pixels crate) have been _removed from the workspace_. I (Cynthia) am no longer fixing bugs or updating code to the latest version of Asterism in those games.

<!-- some of the text here for earlier prototypes is originally from a writeup for a demo from Summer 2020, by Cynthia Li and Julie Ye: https://pom-itb-gitlab01.campus.pomona.edu/cxla2018/asterism-demo. -->

## Paddles Variations
- `paddles-macroquad`: Paddles rewrite with Macroquad for rendering to move away from framebuffer graphics.
- `air_hockey`: Paddles, but flipped 180 degrees and with both horizontal and vertical movement
- `apple-catching`: you control a basket that's catching apples that fall from the sky. Logics used: control, physics, collision, entity state, resource.
- `breakout`: remake of Atari Breakout. Trans rights :)
- `trick-ball`: when the ball hits a paddle, it slows down, and when it hits the top and bottom walls, it speeds up.
- `trick-paddle`: when the ball hits a paddle, the paddle speeds up.
- `pinball`: there are also a bunch of walls between the paddles that you can bounce off.
- `wall-breaker`: like Pinball, but you also break the walls when you hit them.
- `paddle-ball-mania`: multiple balls are in play at all times.
- `maze-macroquad`: rewrite of `maze-minigame` to use Macroquad + Asterism controls and physics. Logics: collision, resource, linking, physics, control.

