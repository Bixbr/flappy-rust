# Building Flappy Bird (Rust port)

## Build matrix

| Platform | Compilation | Packaging | Runtime |
|---|---|---|---|
| Linux | Verified | Native binary | Verified |
| Web | Verified (wasm32) | Partial | Not verified |
| Android | Verified (cargo-ndk) | Verified (assembleRelease) | Not verified |
| Wii U | Not verified | Not verified | Not verified |
| Vita | Partial (minimal binary) | Not verified | Not verified |

## Prerequisites

- Rust toolchain (`rustup`) with the following targets available:
  - `aarch64-linux-android` (Android)
  - `x86_64-linux-android` (Android emulator)
  - `wasm32-unknown-unknown` (Web)
- Macroquad dependencies installed
- Assets in `assets/` directory (atlas.png, atlas.txt, sfx_*.ogg)

---

## 1. Linux standalone executable

Build and run the native Linux version:

```bash
cargo run -p flappy-app
```

Or build a release binary:

```bash
cargo build -p flappy-app --release
```

The binary will be at `target/release/flappy-app`.

---

## 2. HTML (WebAssembly)

Build the WASM binary and generate the HTML wrapper:

```bash
cargo build -p flappy-app --target wasm32-unknown-unknown
```

Copy the generated files to a web server directory:

```bash
# The WASM file and JS glue will be in target/wasm32-unknown-unknown/release/
# Copy flappy_app.wasm and flappy_app.js to your web server
# Use the index.html template at platforms/web/index.html
```

Open `index.html` in a browser. The game requires a user gesture to start audio (browser autoplay rule).

---

## 3. Android APK (work in progress)

Build the native shared libraries for Android:

```bash
# Install cargo-ndk if not already:
# cargo install cargo-ndk

# Build native libraries for arm64-v8a (primary) and x86_64 (emulator):
cargo ndk -t arm64-v8a -t x86_64 -o android/app/src/main/jniLibs build -p flappy-app --release
```

The following will be produced:
- `android/app/src/main/jniLibs/arm64-v8a/libflappy_app.so`
- `android/app/src/main/jniLibs/arm64-v8a/liblewton-*.so`
- `android/app/src/main/jniLibs/x86_64/libflappy_app.so`
- `android/app/src/main/jniLibs/x86_64/liblewton-*.so`

Copy assets to the Android project:

```bash
# Assets are already in android/app/src/main/assets/ from the fix above:
# atlas.png, atlas.txt, sfx_wing.ogg, sfx_point.ogg, sfx_hit.ogg, sfx_swooshing.ogg
```

Build the release APK:

```bash
cd android
./gradlew assembleRelease
```

The APK will be at `android/app/build/outputs/release/app-release.apk`.

**Note**: Full APK packaging requires the Android SDK and NDK properly configured in the environment. The core native library build works with `cargo-ndk` as shown above.

---

## 4. PlayStation Vita VPK (work in progress)

Build for Vita using the rust-vita target:

```bash
# Target is armv7-sony-vita-newlibeabihf
cargo build -p flappy-core --target armv7-sony-vita-newlibeabihf
```

Vita-specific frontend requires:
- VitaSDK installed (at `/opt/vitasdk/` typically)
- `cargo-vita` installed
- A frontend using `vita2d` or `VitaGL` for sprites and `sceAudio` for sound

Package as VPK:

```bash
cargo vita package -p flappy-app --target armv7-sony-vita-newlibeabihf
```

The VPK will be placed in the vita output directory.

**Note**: Vita toolchain not fully available in this environment. See RECAP.md for current status.

---

## 5. Wii U WUI/Wuhb (work in progress)

Build for Wii U using devkitPro toolchain:

```bash
# Requires devkitPro: powerpc-eabi-gcc, devkitPPC, WUT
# The community Rust target or a C/C++ launcher hosting the Rust library is needed
```

Rendering uses GX2, input uses VPAD, audio uses AX engine.

Package as `.rpx` or `.wuhb`.

**Note**: devkitPro/dekitPPC not available in this environment. See RECAP.md for current status.