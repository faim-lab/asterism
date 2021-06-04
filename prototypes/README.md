# Prototypes

These are some prototypes of games made with `asterism`.

## Paddles Variations

Paddles variations, mostly using control, physics, resource, and collision logics.

- `paddles-macroquad`: Paddles rewrite with Macroquad for rendering to move away from framebuffer graphics. Player 1: Q - up, A - down, W - server; player 2: O - up, L - down, I - serve.
- `breakout`: remake of Atari Breakout. Left and right to move the paddle, space to serve. Trans rights :)
- `trick-ball`: when the ball hits a paddle, it slows down, and when it hits the top and bottom walls, it speeds up.
- `trick-paddle`: when the ball hits a paddle, the paddle speeds up.
- `wall-breaker`: Paddles but there are walls, and you break the walls when you hit them.
- `paddle-ball-mania`: multiple balls are in play at all times.
- `maze-macroquad`: rewrite of `maze-minigame` to use Macroquad + Asterism controls and physics. Arrow keys to move around. Additional logics: linking.
- `apple-catching`: you control a basket that's catching apples that fall from the sky. Left and right to move the basket. Additional logics used: entity state.
- `omnipaddles`: game that attempts to generalize a game state across a few similar paddles variations.

## Rendering and Communication Channels

These games use a json file to define sprite animations, which play based on information given by logics.

- `clowder`: game about herding cats
- `apple-catching`
- other stuff Katiana's working on

## Engines

These engines attempt to compose logics to form an engine rather than an individual game. Engines provide types that represent mappings across logics. We aim to simplify the process of building usable engines through `asterism` (eventually through proc macros?).

- `paddles-engine`: offers the types Paddle (control, collision), Wall (collision), Ball (collision, physics), Score (resource). Attempts to make the logics own all game state data. Examples: paddles, breakout (run with ex. `cargo run --bin paddles`)

# Early Prototypes

- `yarn` (or, _sick day blues_): text game about being sick.

# Unsupported prototypes

- `archer_game`: A game made with the WGPU. Doesn't use Asterism. I recommend not running this because compiling it will take seven minutes (ergo it is not included in the workspace).

The ultraviolet crate is no longer supported by `asterism`, so these no longer run:

- `paddles`: remake of Atari Pong. Outputs to a framebuffer using the Pixels crate, as well as the terminal. Logics used: control, physics, collision, resource.
- `jumper`: simple 2d platformer. Outputs to a framebuffer using the Pixels crate. Logics used: control, physics, collision, entity-state.
- `maze-minigame`: top-down navigation game. Outputs to a framebuffer using the Pixels crate. Has some destroyable items and portals (from Portal 2). Logics used: collision, resource, linking. Control and physics aren't implemented with Asterism.
- `air_hockey`: Paddles, but flipped 180 degrees and with both horizontal and vertical movement. Will not run; see `clowder`.

<!-- some of the text here for earlier prototypes is originally from a writeup for a demo from Summer 2020, by Cynthia Li and Julie Ye: https://pom-itb-gitlab01.campus.pomona.edu/cxla2018/asterism-demo. -->
