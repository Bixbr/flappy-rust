# Flappy Bird — Rust port

A faithful port of the decompiled Flappy Bird 1.3 APK using
[macroquad](https://macroquad.rs/). The original 288×512 coordinate space,
60 Hz frame-based physics, MWC random generator, pipe layout, collision boxes,
state machine, score panel and medal thresholds are reproduced from the
original APK. Ads, Google Play and leaderboard integrations are intentionally
left out.
## Important

**No copyrighted material** is provided in the source code, In order to build the game you will NEED to provide your own assets.

## Controls

- Desktop / Web: click, tap or press Space.
- Vita: Cross, Start, Up or R.
- Wii U: A, Start, D-pad Up or R.

## Releases

Can be found on the github releases page.

## Build

Requirements: Rust stable, plus the platform toolchains listed below. Run
`cargo test -p flappy-core` for the shared game-logic tests.

### Linux / desktop

```sh
cargo build --release -p flappy-app
./target/release/flappy-app
```

### Web

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown -p flappy-app
cp target/wasm32-unknown-unknown/release/flappy_app.wasm platforms/web/
python3 -m http.server -d platforms/web 8000
```

Open `http://localhost:8000`. The `.wasm` file must be served over HTTP; opening
`index.html` from disk will not work.

### Android

Requirements: Android SDK, NDK, `cargo-ndk`.

```sh
echo "sdk.dir=$ANDROID_HOME" > android/local.properties
cargo ndk -t arm64-v8a -t x86_64 -o android/app/src/main/jniLibs build -p flappy-app --release
(cd android && ./gradlew assembleRelease)
```

The unsigned APK is written to
`android/app/build/outputs/apk/release/app-release-unsigned.apk`. Sign it before
installing or distributing:

```sh
$ANDROID_HOME/build-tools/34.0.0/apksigner sign \
  --ks my-release-key.jks \
  --out releases/flappy-android.apk \
  android/app/build/outputs/apk/release/app-release-unsigned.apk
```

### PlayStation Vita

Requirements: VitaSDK, `cargo-vita`, nightly Rust with `rust-src`.

```sh
cd platforms/vita
cargo vita build vpk -- --release
```

The VPK is written to
`platforms/vita/target/armv7-sony-vita-newlibeabihf/release/flappy-vita.vpk`.
Copy it to the Vita and install it with VitaShell.

### Wii U

Requirements: devkitPro with devkitPPC and WUT, the Wii U SDL2 portlibs
(`~/wiiu-portlibs` by default), nightly Rust with `rust-src`, and ImageMagick.

```sh
./platforms/wiiu/build.sh
```

The script writes `releases/flappy-wiiu.wuhb` and `releases/flappy-wiiu.rpx`.
Place the WUHB in `sd:/wiiu/apps/flappy-rust/` and launch it from the Homebrew
Launcher.

## Layout

- `crates/flappy-core` — platform-independent game logic, RNG and atlas parser.
- `crates/flappy-app` — macroquad desktop/web/Android frontend.
- `crates/flappy-wiiu` — C ABI used by the Wii U frontend.
- `platforms/` — Vita, Wii U and web entry points.
- `assets/` — sprites and sounds extracted from the original APK.
