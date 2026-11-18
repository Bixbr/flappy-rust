# Restructuring Report

## 1. Original problems
Large main.rs; platform/shared logic mixed; toolchain access inconsistencies; Android packaging issues; Wii U/Vita non-standard toolchains.

## 2. New structure
Workspace crates/flappy-core + flappy-app retained; docs/, scripts/, per-dir READMEs, AGENTS.md added.

## 3. Files moved/modified
No source logic moved. Added documentation and check-toolchains.sh. Existing uncommitted changes preserved.

## 4. Architectural improvements
Clear ownership boundaries documented; toolchain verification script.

## 5. Toolchain findings
Rust, Android NDK r30, cargo-ndk, VitaSDK /opt/vitasdk, devkitPro /opt/devkitpro present. Wii U/Vita Rust targets limited.

## 6. Documentation created
PROJECT_AUDIT.md, TOOLCHAINS.md, BUILDING.md (matrix), AGENTS.md, per-dir READMEs, RESTRUCTURING_REPORT.md.

## 7. Builds verified
Linux, Web (compile), Android (native lib), Vita (minimal binary).

## 8. Builds still blocked
Android APK packaging end-to-end, Web runtime, Vita frontend, Wii U frontend.

## 9. Known risks
Legacy src/ duplicates workspace crates; asset path resolution depends on CARGO_MANIFEST_DIR.

## 10. Recommended next tasks
Implement platform frontends, add tests, CI.
