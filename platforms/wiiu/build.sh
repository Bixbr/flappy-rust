#!/usr/bin/env bash
# Build the Wii U homebrew application (wut + SDL2 + shared Rust core).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HERE="$ROOT/platforms/wiiu"
OUT="$ROOT/target/wiiu"
SDL="$HOME/wiiu-portlibs/opt/devkitpro/portlibs/wiiu"
TARGET="$HERE/powerpc-unknown-none-eabi.json"

export DEVKITPRO="${DEVKITPRO:-/opt/devkitpro}"
export DEVKITPPC="${DEVKITPPC:-$DEVKITPRO/devkitPPC}"

mkdir -p "$OUT" "$HERE/assets"

if [ ! -f "$HERE/assets/atlas_rgba.bin" ]; then
    magick "$ROOT/assets/atlas.png" -depth 8 RGBA:"$HERE/assets/atlas_rgba.bin"
fi
(cd "$HERE/assets" && powerpc-eabi-objcopy -I binary -O elf32-powerpc -B powerpc \
    atlas_rgba.bin "$OUT/atlas_rgba.o")

cd "$ROOT"
rustup run nightly cargo build -Zjson-target-spec \
    -Zbuild-std=core,compiler_builtins \
    -Zbuild-std-features=compiler-builtins-mem \
    --target "$TARGET" --release -p flappy-wiiu

powerpc-eabi-gcc \
    -specs="$DEVKITPRO/wut/share/wut.specs" \
    -I"$DEVKITPRO/wut/include" -I"$SDL/include" \
    -O2 -Wall -Wextra -D__WIIU__ -D__WUT__ \
    "$HERE/src/main.c" "$OUT/atlas_rgba.o" \
    "$ROOT/target/powerpc-unknown-none-eabi/release/libflappy_wiiu.a" \
    -L"$SDL/lib" -lSDL2main -lSDL2 \
    -L"$DEVKITPRO/wut/lib" -lwut -lm \
    -o "$OUT/flappy-wiiu.elf"

powerpc-eabi-objcopy --remove-section=.note.GNU-stack \
    "$OUT/flappy-wiiu.elf" "$OUT/flappy-wiiu-clean.elf"
elf2rpl "$OUT/flappy-wiiu-clean.elf" "$OUT/flappy-wiiu.rpx"
wuhbtool "$OUT/flappy-wiiu.rpx" "$OUT/flappy-wiiu.wuhb" \
    --name="Flappy Rust" --short-name="Flappy Rust" --author="flappy-rust"

mkdir -p "$ROOT/builds"
cp "$OUT/flappy-wiiu.rpx" "$ROOT/builds/flappy-wiiu.rpx"
cp "$OUT/flappy-wiiu.wuhb" "$ROOT/builds/flappy-wiiu.wuhb"
echo "built: $ROOT/builds/flappy-wiiu.wuhb"
