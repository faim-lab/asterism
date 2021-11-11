# Prototypes

These are some prototypes of games made with Asterism.

## Engines

These engines attempt to compose logics to form an engine rather than an individual game. Engines provide types that represent mappings (i.e. structural syntheses) across logics. We aim to simplify the process of building usable engines through Asterism (eventually through proc macros?).

- `paddles-engine`: offers the types Paddle (control, collision), Wall (collision), Ball (collision, physics), Score (resource). Attempts to make the logics own all game state data. Examples: paddles, breakout (run with ex. `cargo run --bin paddles`)
- `boxsy`: fake Bitsy. Offers the types Player (control, collision), Resource (resource), and Tile (collision). Allows the user to create rooms and link between locations in rooms. Examples: extreme-dungeon-crawler

## Rendering and Communication Channels

These games use a json file to define sprite animations, which play based on information given by logics. They don't use the engine structure described above. Both are moderately to mildly broken.

- `clowder`: game about herding cats
- `apple-catching`: Kaboom!-like game about catching falling apples

# Unsupported prototypes

The prototypes in the `old/` folder no longer compile.

## Paddles Variations

Paddles variations, mostly using control, physics, resource, and collision logics.

- `paddles-macroquad`: Paddles rewrite with macroquad for rendering to move away from framebuffer graphics. Player 1: Q - up, A - down, W - serve; player 2: O - up, L - down, I - serve.
- `air_hockey`: Paddles, but flipped 180 degrees and with both horizontal and vertical movement. For an updated version of this game, see `clowder`.
- `breakout`: remake of Atari Breakout. Left and right to move the paddle, space to serve. Trans rights :)
- `trick-ball`: when the ball hits a paddle, it slows down, and when it hits the top and bottom walls, it speeds up.
- `trick-paddle`: when the ball hits a paddle, the paddle speeds up.
- `wall-breaker`: Paddles but there are walls, and you break the walls when you hit them.
- `paddle-ball-mania`: multiple balls are in play at all times.
- `maze-macroquad`: rewrite of `maze-minigame` to use macroquad + asterism controls and physics. Arrow keys to move around. Additional logics: linking.
- `apple-catching`: you control a basket that's catching apples that fall from the sky. Left and right to move the basket. Additional logics used: entity state.
- `omnipaddles`: game that attempts to generalize a game state across a few similar paddles variations.

## Early Prototypes

- `yarn` (or, _sick day blues_): text game about being sick.
- `paddles`: remake of Atari Pong. Outputs to a framebuffer using the Pixels crate, as well as the terminal. Logics used: control, physics, collision, resource.
- `jumper`: simple 2d platformer. Outputs to a framebuffer using the Pixels crate. Logics used: control, physics, collision, entity-state.
- `maze-minigame`: top-down navigation game. Outputs to a framebuffer using the Pixels crate. Has some destroyable items and portals (from Portal 2). Logics used: collision, resource, linking. Control and physics aren't implemented with asterism.
- `archer_game`: A game made with the WGPU. Doesn't use asterism. I recommend not running this because compiling it will take seven minutes.


<!-- some of the text here for earlier prototypes is originally from a writeup for a demo from Summer 2020, by Cynthia Li and Julie Ye: https://pom-itb-gitlab01.campus.pomona.edu/cxla2018/asterism-demo. -->
