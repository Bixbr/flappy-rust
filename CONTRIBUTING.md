# Contributing

Thanks for helping improve the Flappy Bird Rust port. These guidelines keep the
history readable and the codebase healthy. Use British English in prose,
documentation and comments.

## Getting started

1. Fork the repository and create a topic branch.
2. Install a current stable Rust toolchain.
3. Install the native build dependencies for your platform. On Debian or Ubuntu:

   ```sh
   sudo apt install build-essential pkg-config libasound2-dev libx11-dev \
       libxi-dev libgl1-mesa-dev libudev-dev
   ```

4. Run the game with `cargo run --release`.

## Commit messages

Use the following format:

```text
[Topic] Imperative summary
```

- Start with a bracketed topic.
- Use the imperative mood: “Add”, “Fix”, “Remove”, “Refactor”, not “Added” or “Fixes”.
- Use sentence case and do not end the subject with a full stop.
- Keep the subject to 72 characters or fewer.
- Make one logical change per commit.
- Explain *why* in the body when the subject alone is not enough. Wrap the body at 72 characters.
- Reference issues at the end of the body, for example `Refs: #42`.

### Topics

| Topic | Use for |
| --- | --- |
| `[Android]` | Android tooling, Gradle, APK and AAB packaging |
| `[Web]` | WebAssembly, browser packaging and JavaScript glue |
| `[Linux]` | Native Linux builds and packaging |
| `[Vita]` | PlayStation Vita builds and backend work |
| `[Wii U]` | Wii U builds and backend work |
| `[Gameplay]` | Physics, collisions, scoring and state machine |
| `[Graphics]` | Atlas loading, sprites, cameras and scaling |
| `[Audio]` | Sound loading, mixing and platform audio |
| `[Input]` | Keyboard, mouse, touch and controller input |
| `[Assets]` | Game assets and their metadata |
| `[Build]` | Cargo configuration, scripts and dependencies |
| `[CI]` | Continuous integration and releases |
| `[Docs]` | Documentation, README and guides |
| `[Repo]` | Repository housekeeping and tooling |
| `[Chore]` | Small maintenance changes that fit nowhere else |

### Examples

```text
[Android] Add 64-bit APK build support
[Web] Package the game with wasm-bindgen
[Gameplay] Match pipe recycling to the original engine
[Graphics] Load atlas regions from texture dimensions
[Docs] Explain widescreen camera behaviour
[Repo] Add editor configuration and commit guide
```

### Commit hygiene

- Do not commit build output, generated binaries, local save data or secrets.
- Keep commits buildable. Run the checks below before pushing.
- Avoid “WIP”, “fix”, “stuff” and similar messages.
- Rebase or squash work-in-progress commits before opening a pull request.
- Do not rewrite shared history unless the team has agreed to it.

## Branches

Use a short topic prefix and a lower-case summary with hyphens:

```text
android/apk-build
web/wasm-packaging
graphics/hd-atlas
fix/pipe-scoring
docs/widescreen-camera
```

Keep branches focused. Rebase onto `main` before opening a pull request.

## Code style

- Format Rust with `cargo fmt`.
- Treat Clippy warnings as errors where practical:

  ```sh
  cargo clippy --all-targets -- -D warnings
  ```

- Keep game logic independent of rendering and platform code where possible.
- Do not add comments that merely restate the code. Only section headings are
  used in the source; put longer explanations in this document or in the
  relevant design notes.
- Preserve the original 288×512 coordinate system and 60 Hz simulation unless a
  change explicitly targets that behaviour.
- Prefer descriptive names over abbreviations.

## Checks before pushing

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

If you cannot run a platform-specific build, say so in the pull request.

## Pull requests

- Describe the problem and the approach.
- Include screenshots or recordings for visible changes.
- Note any platform that needs testing.
- Keep unrelated refactors out of the same pull request.
- Update the README or design documents when behaviour changes.
- Be responsive to review; push follow-up commits rather than force-pushing
  during an active review unless asked otherwise.

## Assets and licensing

The game assets originate from the Flappy Bird APK and remain the property of
their original owner. Do not add assets that you do not have permission to
distribute. Record the source and licence of any new asset in the pull request.
