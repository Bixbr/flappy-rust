fn main() {
    println!("cargo:rustc-link-search=native=/opt/vitasdk/arm-vita-eabi/lib");
    println!("cargo:rustc-link-lib=static=vita2d");
    println!("cargo:rustc-link-lib=static=png");
    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=SceCtrl_stub");
    println!("cargo:rustc-link-lib=dylib=SceDisplay_stub");
    println!("cargo:rustc-link-lib=dylib=SceGxm_stub");
    println!("cargo:rustc-link-lib=dylib=SceAppMgr_stub");
    println!("cargo:rustc-link-lib=dylib=SceCommonDialog_stub");
    println!("cargo:rustc-link-lib=dylib=SceTouch_stub");
    println!("cargo:rustc-link-lib=dylib=ScePower_stub");
}