# Toolchains

## Rust
- rustup default toolchain; targets: aarch64-linux-android, x86_64-linux-android, wasm32-unknown-unknown, armv7-sony-vita-newlibeabihf (nightly).

## Android
- SDK: `$ANDROID_HOME` / `$ANDROID_SDK_ROOT`
- NDK: r30 (check `/opt/android-ndk` or `$ANDROID_NDK_HOME`)
- cargo-ndk: cargo install cargo-ndk

## Web
- wasm32-unknown-unknown target; wasm-bindgen (optional).

## Vita
- VitaSDK: /opt/vitasdk
- cargo-vita installed
- nightly + rust-src

## Wii U
- devkitPro: /opt/devkitpro (powerpc-eabi-gcc, devkitPPC, WUT)

## Environment variables
- PATH must include toolchain bin dirs.
- ANDROID_HOME, ANDROID_NDK_HOME, VITASDK, DEVKITPRO.

## Permissions
- Toolchains installed system-wide under /opt; readable by all users. Per-user cargo in ~/.cargo/bin.

## Known limitations
- No Rust target for Wii U powerpc-eabi or Vita arm-vita-eabi (Rust target is armv7-sony-vita-newlibeabihf).
- Gradle builds require SDK/NDK configured.
