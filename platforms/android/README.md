# Android build (pending toolchain)

Requirements: Android SDK, NDK, cargo-ndk, Gradle.

Steps:
1. rustup target add aarch64-linux-android x86_64-linux-android
2. cargo ndk -t arm64-v8a -t x86_64 -o app/src/main/jniLibs build -p flappy-app --release
3. Place assets in app/src/main/assets
4. ./gradlew assembleRelease
