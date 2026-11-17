# Flappy Bird — Rust port

A faithful port of the decompiled Flappy Bird 1.3 APK to Rust using
[macroquad](https://macroquad.rs/). The original 288×512 coordinate space,
frame-based physics (60 Hz), MWC random generator, pipe layout, collision
boxes, state machine, score panel and medal thresholds are reproduced from
the orginal Flappy Bird APK

## Run

```sh
cargo run --release
```

The window is letterboxed to the original 9:16 aspect ratio.

## Controls

- The same from before.

## Assets

All sprites and sounds come from the APK:

- `assets/atlas.png`, `assets/atlas.txt`
- `assets/sfx_*.ogg`

## Layout

- `src/atlas.rs` — atlas.txt parser and sprite renderer
- `src/rng.rs` — exact `com.dotgears.j` random generator
- `src/main.rs` — game loop, physics, collisions, scoring, UI

The ad / Google Play / leaderboard integrations from the APK are intentionally
left out, the corresponding menu buttons are now useless.
