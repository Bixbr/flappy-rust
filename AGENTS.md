# AGENTS.md

## Architecture
Workspace: flappy-core (pure logic/RNG), flappy-app (Macroquad frontend). Shared game logic must stay in flappy-core where possible; platform code in platforms/.

## Coding standards
- Rust 2021, cargo fmt (max_width 100).
- British English in docs.
- Keep 288x512 coordinate system and 60 Hz simulation unchanged.

## Build verification
Run cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test, cargo build --release before pushing.

## Git
Topic branches, commit format `[Topic] Imperative summary`. No destructive ops. No commits without authorisation.

## Assets
Do not delete or modify assets.

## Toolchains
Use installed toolchains at documented paths; do not install duplicates. Run scripts/check-toolchains.sh.

## Platform ownership
Android agent owns android/. Vita agent owns platforms/vita. Wii U owns platforms/wiiu. Web owns platforms/web. Linux owns native. Game physics changes require review.

## Reporting failures
Document uncertainties rather than guessing. Never claim a build succeeded unless verified.
