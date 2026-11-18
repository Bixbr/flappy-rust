# Project Audit

## 1. Summary

Flappy Bird Rust port: faithful reconstruction of Flappy Bird 1.3 from a decompiled APK, using Macroquad. Workspace split into `flappy-core` (RNG) and `flappy-app` (Macroquad frontend). Platforms: Linux (verified), Web (compile verified), Android (native lib verified), Vita (toolchain partially verified, no frontend), Wii U (toolchain available, no frontend).

## 2. Directory tree (significant files)

- `Cargo.toml` / `Cargo.lock` — workspace (flappy-core, flappy-app)
- `crates/flappy-core/` — pure Rust, RNG only. `src/lib.rs`, `src/rng.rs`
- `crates/flappy-app/` — Macroquad frontend. `src/main.rs` (game loop, physics, rendering, UI), `src/atlas.rs` (atlas parser/renderer), `src/lib.rs`
- `src/` — legacy single-crate sources (main.rs, atlas.rs, rng.rs) superseded by workspace crates; retained.
- `assets/` — atlas.png, atlas.txt, sfx_*.ogg
- `android/` — Gradle project, assets, jniLibs
- `platforms/` — web index.html; android/vita/wiiu READMEs
- `dist/` — web build output (gitignored-ish but present)
- `docs/` — BUILDING.md
- `scripts/` — none yet

## 3. Game logic locations

All gameplay lives in `crates/flappy-app/src/main.rs` (~1339 lines): physics, collisions, scoring, state machine, UI. RNG in `crates/flappy-core/src/rng.rs`. Atlas in `crates/flappy-app/src/atlas.rs`.

## 4. Platform-specific

- Android: `android/` Gradle + jniLibs
- Web: `platforms/web/index.html`, wasm target
- Vita: toolchain only
- Wii U: toolchain only
- Linux: native

## 5. Dependencies

Macroquad (rendering/audio/input), flappy-core (RNG). Workspace resolver 2.

## 6. Incomplete implementations

- Vita frontend not implemented.
- Wii U frontend not implemented.
- Best score in-memory only.
- HD textures / widescreen not implemented.

## 7. Verified builds

- Linux: cargo build/check verified.
- Web: wasm32 build verified.
- Android: cargo-ndk arm64/x86_64 verified.
- Vita: minimal binary compile verified.
- Wii U: toolchain present, no Rust target.

## 8. Toolchain locations

- Rust: rustup (default)
- Android SDK/NDK: present (NDK r30)
- cargo-ndk: installed
- VitaSDK: /opt/vitasdk
- devkitPro: /opt/devkitpro
- wasm-bindgen: installed

## 9. Uncertainties

- Gradle assembleRelease not verified end-to-end in this environment.
- Legacy `src/` duplicates workspace crates; not removed to preserve functionality.
