# Recap and Technical Plan

This document records what has been built, what has been verified, and how the
next stages — HD textures, widescreen presentation and additional platforms —
should be approached.

## 1. What was done

### APK inspection and extraction

- Located the Flappy Bird 1.3 APK through the RDX MCP instance.
- Exported and decompiled all 26 `com.dotgears` classes. `jadx` output was used
  as the authoritative source for game logic.
- Extracted the following assets into `assets/`:
  - `atlas.png`
  - `atlas.txt`
  - `sfx_wing.ogg`, `sfx_point.ogg`, `sfx_hit.ogg`, `sfx_die.ogg`,
    `sfx_swooshing.ogg`
- Ignored the ad, Google Play Games and other third-party SDK packages.

### Reverse-engineered behaviour

The port reproduces the original:

- 288×512 coordinate system and 60 Hz frame-based simulation.
- Bird physics: gravity acceleration, terminal fall speed, flap impulse,
  rotation velocity and rotation acceleration with clamping.
- Bird hitbox, sprite offsets, ground clamping and menu bobbing.
- Pipe spacing, widths, gaps, initial positions, recycling and random heights.
- MWC random generator, including seeding, modulo range handling and the exact
  call order for bird colour, background choice and pipe heights.
- Inclusive AABB collisions for both pipes and the ground.
- Score trigger at the bird's leading edge.
- State machine: title menu, fade transitions, ready overlay, gameplay, death,
  game-over text, score panel and restart.
- Score panel slide, count-up, new-best badge, medal thresholds and medal
  sparkles.
- Original bitmap font rendering, including centring, leading-zero suppression
  and digit advances.
- Tween easing types used by the original game.
- Sound triggers for wing, point, hit, die and swoosh events.

### Rust implementation

- `src/main.rs` — fixed-step game loop, simulation, input, rendering and UI.
- `src/atlas.rs` — `atlas.txt` parser and sprite drawing.
- `src/rng.rs` — the exact `com.dotgears.j` generator.
- Macroquad provides the window, texture and audio backends.
- Rendering uses a 288×512 render target, then scales it uniformly into the
  window with letterboxing.
- Audio support is enabled through Macroquad's `audio` feature.
- Ad, Google Play Games, leaderboard and store integrations are intentionally
  omitted. The corresponding menu buttons are inert.

### Verification

- `cargo check` passes with no errors or warnings.
- `cargo build` passes.
- The binary launches and was checked visually at the menu, ready screen,
  gameplay and game-over panel.
- Frame captures confirmed sprite layout, pipe gaps, HUD, score panel, medals,
  sparkles and button placement.
- A count-up defect in the score panel was found during visual testing and
  fixed.

### Current limitations

- Best score is held in memory only and is lost when the process exits.
- There is no pause screen in the original build path that was ported.
- Leaderboard, advertisements and store links are not implemented.
- Only the native Linux desktop path is currently configured.

## 2. HD texture support

The goal is to support higher-resolution artwork without changing layout,
collision boxes, animation timing or physics.

### Design

1. Separate logical sprite metadata from texture pixels.
   - `atlas.txt` already stores logical `w` and `h` values plus normalised
     `u`, `v`, `uw` and `uh` coordinates.
   - The atlas loader must stop assuming a 1024×1024 texture. Instead, read the
     actual texture dimensions and calculate the source rectangle as:
     - `x = u × texture_width`
     - `y = v × texture_height`
     - `w = uw × texture_width`
     - `h = uh × texture_height`
   - Logical `w` and `h` remain the values used by all game code.

2. Add an asset-resolution selector.
   - Supported packs could be `atlas@1x.png`, `atlas@2x.png` and
     `atlas@4x.png`, all using the same `atlas.txt` and sprite names.
   - Choose the pack from a user setting or from the display scale, with a
     documented fallback order.
   - Keep the 1× pack as the guaranteed fallback.

3. Render at native resolution.
   - The camera still describes a 288×512 logical world.
   - Sprite destination size is `logical_size × render_scale`.
   - Use nearest-neighbour filtering for the original pixel art and linear
     filtering (with mipmaps where appropriate) for hand-drawn HD art.
   - Prevent atlas bleeding by using padded or extruded sprites when packing.

4. Keep the simulation untouched.
   - No physics, collision or animation constant refers to texture pixels.
   - Non-integer render scales should not feed back into simulation values.

5. HD asset production.
   - Recreate or upscale assets at 2×, 3× and 4×.
   - Keep sprite names, ordering, trim values and normalised coordinates
     identical to the original atlas.
   - Add a build check that every sprite named in `atlas.txt` exists in every
     texture pack.

## 3. Widescreen support

The goal is to show more of the world horizontally — especially pipes ahead —
without stretching sprites and without changing a single physics value.

### Camera model

- Keep the simulation in its original 288×512 coordinate space.
- Fix the vertical scale to the window height: `pixels_per_unit = screen_height
  / 512`. The visible width in world units is then `screen_width /
  pixels_per_unit`.
- If the visible width is less than 288, pillarbox the view. Never scale the x
  and y axes independently.
- If the visible width is greater than 288, keep the original play lane at its
  native scale and expose the extra width as scenery.
- Anchor the play lane so most of the extra space appears to the right. The
  bird lane remains at x = 80 in world coordinates.
- Prefer integer scaling when the window size permits it, and fall back to
  fractional uniform scaling otherwise.

### Scene extension

- Tile the land and background horizontally instead of stretching them.
- Keep the original 288-unit play lane authoritative for collisions, scoring
  and the bird's position.
- Extend the renderer with a configurable maximum look-ahead, for example 96 to
  288 extra world units. This prevents an ultrawide monitor from revealing so
  much of the course that the game becomes trivial.

### Pipes ahead without changing the RNG sequence

The original game keeps only three pipe slots. To show pipes ahead, the
renderer needs access to future pipe positions and heights without disturbing
the authoritative simulation.

1. At the start of a round, clone the main RNG into a pipe-preview RNG.
2. Use the preview RNG to generate a queue of future pipe heights in the same
   order the real game will consume them.
3. On each real pipe recycle, advance the main RNG by one draw and take the
   corresponding height from the queue, or verify that both values match.
4. Render queued pipes at extrapolated x positions beyond the current third
   pipe.
5. When a preview pipe reaches the authoritative ring, promote it without
   changing its height or x position.
6. Only the authoritative ring participates in collisions and scoring.

This maintains the original post-round RNG state, so bird variants and
background choices in later rounds remain identical to the original game.

### User interface

- Centre menus and the score panel within the full viewport or within the play
  lane according to a single documented rule.
- Keep the in-game HUD aligned to the play lane so score positioning remains
  predictable.
- Map window and touch coordinates through the same uniform camera transform.

### Tests

- Assert that the visible play lane always has the same pixels-per-unit on both
  axes.
- Assert that bird, pipe and collision values are unchanged at 16:9, 16:10,
  21:9 and 4:3.
- Compare a scripted gameplay trace under widescreen and the original 9:16 view.

## 4. Platform build support

### Shared preparation

Before adding platforms, split the project into:

- `flappy-core` — deterministic game state, physics, collision, scoring, RNG
  and timers with no rendering or platform dependencies.
- `flappy-app` — Macroquad frontend: assets, rendering, audio and input.
- Optional platform crates and packaging directories.

This keeps the simulation testable and allows non-Macroquad backends to reuse
it unchanged.

Add platform-conditional asset paths, save-data locations and audio setup behind
small abstractions rather than scattering `cfg` checks through gameplay code.

### Android (64-bit APK)

- Targets: `aarch64-linux-android` as the required 64-bit target. Add
  `x86_64-linux-android` for emulator testing if desired.
- Build the frontend as a `cdylib` and use the Macroquad Android template with
  an Android Gradle project.
- Use `cargo-ndk` to build native libraries for the selected ABIs and place
  them in the Gradle project's `jniLibs` output.
- Configure `abiFilters` for `arm64-v8a` and optionally `x86_64`.
- Package assets under the Android assets directory and load them through the
  platform asset API rather than a desktop filesystem path.
- Set sensible `minSdkVersion` and `targetSdkVersion`, handle suspend/resume,
  and pause audio when the app loses focus.
- Produce a signed release APK and, for store distribution, an AAB.
- CI: install the Android SDK and NDK, run the Cargo NDK build, then run the
  Gradle release build.

### Web

- Target: `wasm32-unknown-unknown`.
- Build with Cargo, then post-process with `wasm-bindgen` using the `web`
  target.
- Provide an HTML page with a canvas, the Macroquad JavaScript bundle and an
  asset-loading step.
- Serve assets over HTTP; browser builds cannot read arbitrary local files.
- Start audio only after a user gesture, as required by browser autoplay rules.
- Map touch and pointer events to the same tap and button logic as desktop.
- Package `dist/` for static hosting, itch.io or GitHub Pages.
- CI: build WebAssembly and upload the bundled output.

### Linux

- Build natively with `cargo build --release`.
- Document development packages such as ALSA, X11, OpenGL and udev.
- Package as an AppImage, Flatpak, Snap or distribution package.
- Add a `.desktop` entry and icon set.
- CI: build and test on a current Ubuntu LTS image.

### PlayStation Vita

- Rust support comes from the community `rust-vita` target
  `armv7-sony-vita-newlibeabihf`, together with VitaSDK and `cargo-vita`.
- Macroquad and miniquad do not currently provide a maintained Vita backend.
  A Vita build therefore requires a backend for display, input, audio and file
  access.
- Recommended approach:
  - Reuse `flappy-core` unchanged.
  - Implement a small Vita frontend using vita2d or VitaGL for sprites and
    `sceAudio` for sound.
  - Read assets from `ux0:data/flappy-rust/`.
  - Map the touch screen, rear touch pad and face buttons to tap-to-flap.
  - Package the result as a VPK with `cargo-vita`.
- This is a realistic target because the game is 2D, the atlas is small and the
  audio format is Ogg Vorbis.

### Wii U

- There is no stable official Rust target and no Macroquad backend for Wii U.
- The practical route uses devkitPro's `wut` libraries and devkitPPC, with a
  community Rust target or a C/C++ launcher that hosts the Rust library.
- Rendering would use GX2, input would use VPAD, and audio would use the AX
  engine.
- An alternative is an SDL2-based frontend if a suitable Wii U SDL port is
  available for the chosen toolchain.
- Package as an `.rpx` or `.wuhb`.
- This is the highest-effort target. It should follow the core/frontend split
  and the Vita backend, because both prove that the game logic is independent
  of Macroquad.

### Build matrix summary

| Platform | Toolchain | Packaging | Backend work |
| --- | --- | --- | --- |
| Linux | Stable Rust | AppImage, Flatpak or native package | None |
| Web | `wasm32-unknown-unknown` plus `wasm-bindgen` | Static `dist/` | Browser audio and pointer glue |
| Android | Android NDK, `cargo-ndk`, Gradle | APK and AAB | Asset and lifecycle glue |
| PS Vita | `rust-vita`, VitaSDK, `cargo-vita` | VPK | New vita2d/VitaGL frontend |
| Wii U | devkitPro `wut`, devkitPPC | `.rpx` or `.wuhb` | New GX2 frontend |

## 5. Repository changes made now

- Removed source comments except for section headings.
- Added `.gitignore` for Cargo, platform and packaging output, editor state,
  logs and local save data.
- Added `.editorconfig`, `.gitattributes` and `rustfmt.toml`.
- Added `CONTRIBUTING.md` with the `[Topic] Imperative summary` commit format,
  topic list, branch naming, checks and pull-request expectations.
- Initialised the repository and made the initial commit.

## 6. Recommended next steps

1. Add the platform-neutral `flappy-core` split.
2. Refactor the atlas loader to use actual texture dimensions and add HD packs.
3. Introduce the widescreen camera, tiled scenery and pipe preview queue.
4. Add Linux, Web and Android builds first.
5. Add Vita and Wii U backends after the core/frontend split has settled.
6. Add automated trace tests for physics, RNG and widescreen invariance.

## 7. Workspace refactor (cross-platform preparation)

- Project converted to a Cargo workspace with `crates/flappy-core` and
  `crates/flappy-app`.
- `flappy-core` contains the MWC RNG (`Rng`) with no platform dependencies.
- `flappy-app` contains the Macroquad frontend, rendering, audio and input.
- Linux native build verified: `cargo build` and `cargo build --release`.
- Core tests pass: `cargo test -p flappy-core`.

### Build commands

- Core: `cargo test -p flappy-core`
- App (dev): `cargo run -p flappy-app`
- App (release): `cargo build -p flappy-app --release`

### Platform status

- Linux: supported and verified.
- Android: NDK r30 and cargo-ndk installed; `cargo ndk -t arm64-v8a` produces
  libflappy_app.so successfully. Assets are placed in
  `android/app/src/main/assets/`. Full APK packaging via `./gradlew
  assembleRelease` is configured but requires the Android SDK/NDK to be
  properly set up. x86_64 ABI build also available but arm64-v8a is the
  primary target.
- Web: wasm32 target and wasm-bindgen installed; `cargo build -p flappy-app
  --target wasm32-unknown-unknown` succeeds. HTML template at
  `platforms/web/index.html`. Assets must be placed in the same directory as
  the generated WASM JS file for browser loading.
- PlayStation Vita: rust-vita/VitaSDK toolchain not fully available in this
  environment; not built. Requires VitaSDK manual installation and
  `cargo-vita` for VPK packaging.
- Wii U: devkitPro/devkitPPC not available; not built. Requires devkitPro
  toolchain for GX2 rendering and VPAD input.

## 8. Bug fixes and issues resolved

### Black screen on Linux exec

- **Problem**: `cargo run -p flappy-app` panicked with
  `failed to load assets/atlas.png: FileError { kind: IOError(Os { code: 2,
  kind: NotFound, message: "No such file or directory" }), path: "assets/atlas.png" }`
  because the atlas and sound loaders used relative paths that only worked
  when running from the project root directory.
- **Fix**: Updated `crates/flappy-app/src/atlas.rs` and
  `crates/flappy-app/src/main.rs` to resolve the assets directory relative to
  `CARGO_MANIFEST_DIR`, using `Path::new(&manifest_dir).join("../../assets")`
  to reach the workspace-root `assets/` directory from the crate location.

### HTML black screen

- The web build compiles successfully to WASM32. The HTML template at
  `platforms/web/index.html` is in place. Ensure assets are copied alongside
  the generated `flappy_app.js` when deploying to a web server.

## 9. Android build details

- Built with `cargo ndk -t arm64-v8a -o app/src/main/jniLibs build -p flappy-app --release`
  producing:
  - `android/app/src/main/jniLibs/arm64-v8a/libflappy_app.so`
  - `android/app/src/main/jniLibs/arm64-v8a/liblewton-41a11398c3b9cce1.so`
  - `android/app/src/main/jniLibs/x86_64/libflappy_app.so` (secondary)
- Assets copied to `android/app/src/main/assets/`:
  - atlas.png, atlas.txt, sfx_wing.ogg, sfx_point.ogg, sfx_hit.ogg, sfx_swooshing.ogg
- Gradle `assembleRelease` can be run with `./gradlew assembleRelease`
  from the android directory.
- Primary ABI: `arm64-v8a`. Secondary: `x86_64` for emulator testing.

## 10. Vita build status

- Rust target `armv7-sony-vita-newlibeabihf` is available and compiles
  successfully with `-Z build-std=core,alloc,panic_abort`.
- VitaSDK is installed at `/opt/vitasdk`.
- `cargo-vita` is installed.
- A minimal Rust binary has been compiled for Vita, but no Vita2D frontend
  implementation exists yet. Would require implementing a frontend using
  vita2d or VitaGL for sprites and `sceAudio` for sound, with assets read
  from `ux0:data/flappy-rust/`.

## 11. Wii U build status

- No stable official Rust target and no Macroquad backend for Wii U.
- Practical route uses devkitPro's `wut` libraries and devkitPPC.
- Neither devkitPro nor devkitPPC are available in this environment.
- Would require implementing a GX2 frontend, VPAD input, and AX audio
  backend. Package as `.rpx` or `.wuhb`.

## 12. Repository cleanup

- No temporary build artifacts outside of `target/` and `android/app/src/main/jniLibs/`
  (these are expected build outputs).
- Assets in `assets/` are the required game assets extracted from the original
  APK and should be kept.
- `dist/` directory is empty; web build outputs should be placed there for
  hosting.
- No `.tmp` or `.bak` files found.


## Toolchain installation notes

- VitaSDK AUR build failed during prepare() (patch hunk failure); manual installation required.
- devkitPro not installed; AUR packages unavailable/broken in this environment.


## Native toolchain verification

- devkitPro installed at /opt/devkitpro; `powerpc-eabi-gcc` verified; WUT links successfully.
- VitaSDK installed at /opt/vitasdk; `arm-vita-eabi-gcc` verified.
- No Rust target exists for powerpc-eabi or arm-vita-eabi in this toolchain; native Wii U/Vita frontends must be implemented in C/C++ (or a Rust core compiled via a compatible target, which is not currently available). flappy-core remains reusable as pure logic but cannot be compiled directly for these targets without a Rust target or FFI bridge.


## Vita toolchain verification

- Nightly Rust + rust-src installed.
- Target `armv7-sony-vita-newlibeabihf` builds successfully with `-Z build-std=core,alloc,panic_abort`.
- cargo-vita installed.
- VitaSDK verified.
- Minimal Rust binary compiled for Vita.
