#!/usr/bin/env bash
set -u
check() {
  local name="$1"; shift
  if command -v "$1" >/dev/null 2>&1; then echo "OK   $name: $(command -v "$1")"; else echo "MISS $name: not on PATH"; fi
}
echo "== Rust =="
check rustc rustc
check cargo cargo
rustup show active-toolchain 2>/dev/null || echo "rustup not found"
echo "== Android =="
check cargo-ndk cargo-ndk
echo "ANDROID_HOME=${ANDROID_HOME:-<unset>}"
echo "ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-<unset>}"
ls /opt/android-ndk 2>/dev/null && echo "NDK dir present" || echo "NDK dir not found at /opt/android-ndk"
echo "== Web =="
rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown && echo "OK wasm32 target installed" || echo "MISS wasm32 target"
check wasm-bindgen wasm-bindgen
echo "== Vita =="
echo "VITASDK=${VITASDK:-<unset>}"
ls /opt/vitasdk 2>/dev/null && echo "VitaSDK dir present" || echo "VitaSDK dir not found"
check cargo-vita cargo-vita
echo "== Wii U =="
echo "DEVKITPRO=${DEVKITPRO:-<unset>}"
ls /opt/devkitpro 2>/dev/null && echo "devkitPro dir present" || echo "devkitPro dir not found"
command -v powerpc-eabi-gcc >/dev/null 2>&1 && echo "OK powerpc-eabi-gcc" || echo "MISS powerpc-eabi-gcc"
