#![cfg_attr(target_os = "none", no_std)]

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use flappy_core::game::{Game, Point};
use flappy_core::{AtlasMap, SpriteRect};

struct Global<T>(UnsafeCell<MaybeUninit<T>>);

unsafe impl<T> Sync for Global<T> {}

static GAME: Global<Game> = Global(UnsafeCell::new(MaybeUninit::uninit()));
static ATLAS: Global<AtlasMap> = Global(UnsafeCell::new(MaybeUninit::uninit()));

fn game() -> &'static mut Game {
    unsafe { &mut *(*GAME.0.get()).as_mut_ptr() }
}

fn atlas() -> &'static AtlasMap {
    unsafe { &*(*ATLAS.0.get()).as_ptr() }
}

#[no_mangle]
pub extern "C" fn fw_init(seed: i32) {
    unsafe {
        (*GAME.0.get()).write(Game::new(seed));
        (*ATLAS.0.get()).write(AtlasMap::parse(include_str!("../../../assets/atlas.txt")));
    }
}

#[no_mangle]
pub extern "C" fn fw_tick() {
    game().tick();
}

#[no_mangle]
pub extern "C" fn fw_tap() {
    game().handle_input(true, false, Point::new(0.0, 0.0));
}

#[no_mangle]
pub extern "C" fn fw_tap_at(x: f32, y: f32) {
    game().handle_input(true, false, Point::new(x, y));
}

#[no_mangle]
pub extern "C" fn fw_release_at(x: f32, y: f32) {
    game().handle_input(false, true, Point::new(x, y));
}

#[no_mangle]
pub extern "C" fn fw_activate() {
    game().activate_play();
}

#[repr(C)]
pub struct SpriteRectOut {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn fw_sprite_rect(
    name: *const u8,
    len: usize,
    out: *mut SpriteRectOut,
) -> i32 {
    let bytes = unsafe { core::slice::from_raw_parts(name, len) };
    let Ok(name) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let rect: SpriteRect = atlas().rect(name);
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return 0;
    }
    unsafe {
        *out = SpriteRectOut {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
        };
    }
    1
}

macro_rules! getters {
    ($($name:ident -> $ty:ty, $field:expr;)*) => {
        $(
            #[no_mangle]
            pub extern "C" fn $name() -> $ty {
                let g = game();
                $field(g)
            }
        )*
    };
}

getters! {
    fw_menu -> i32, |g: &Game| g.menu as i32;
    fw_bg_night -> i32, |g: &Game| g.bg_night as i32;
    fw_bird_x -> i32, |g: &Game| g.bird.x;
    fw_bird_y -> i32, |g: &Game| g.bird.y;
    fw_bird_rot -> f32, |g: &Game| g.bird.rot;
    fw_bird_bob -> f32, |g: &Game| g.bird.bob_y;
    fw_bird_variant -> i32, |g: &Game| g.bird.variant;
    fw_bird_frame -> i32, |g: &Game| g.bird.frame_index();
    fw_bird_bobbing -> i32, |g: &Game| g.bird.bobbing as i32;
    fw_land_x -> i32, |g: &Game| g.land_x;
    fw_score -> i32, |g: &Game| g.score;
    fw_best -> i32, |g: &Game| g.best;
    fw_pipe_speed -> i32, |g: &Game| g.pipe_speed;
    fw_pipes_pending -> i32, |g: &Game| g.pipes_pending;
    fw_ready_visible -> i32, |g: &Game| g.ready.visible as i32;
    fw_ready_alpha -> f32, |g: &Game| g.ready.alpha.value;
    fw_over_visible -> i32, |g: &Game| g.over_text.visible as i32;
    fw_over_alpha -> f32, |g: &Game| g.over_text.alpha.value;
    fw_over_offset -> i32, |g: &Game| g.over_text.offset;
    fw_panel_visible -> i32, |g: &Game| g.panel.visible as i32;
    fw_panel_x -> i32, |g: &Game| g.panel.x;
    fw_panel_y -> i32, |g: &Game| g.panel.y;
    fw_panel_display -> i32, |g: &Game| g.panel.display;
    fw_panel_best -> i32, |g: &Game| g.panel.best;
    fw_panel_state -> i32, |g: &Game| g.panel.state;
    fw_panel_new -> i32, |g: &Game| g.panel.new_badge as i32;
    fw_play_visible -> i32, |g: &Game| g.play_visible as i32;
    fw_score_visible -> i32, |g: &Game| g.score_visible as i32;
    fw_rate_visible -> i32, |g: &Game| g.rate_visible as i32;
    fw_fade_value -> f32, |g: &Game| g.fade.value;
    fw_fade_done -> i32, |g: &Game| g.fade.done as i32;
    fw_flash_value -> f32, |g: &Game| g.flash.value;
    fw_flash_done -> i32, |g: &Game| g.flash.done as i32;
}

#[no_mangle]
pub extern "C" fn fw_pipe_x(i: i32) -> i32 {
    game().pipes[i as usize].0
}

#[no_mangle]
pub extern "C" fn fw_pipe_y(i: i32) -> i32 {
    game().pipes[i as usize].1
}

#[no_mangle]
pub extern "C" fn fw_panel_medal() -> i32 {
    match game().panel.medal {
        Some(m) => m as i32,
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn fw_panel_medal_x() -> i32 {
    game().panel.medal_x
}

#[no_mangle]
pub extern "C" fn fw_panel_medal_y() -> i32 {
    game().panel.medal_y
}

#[no_mangle]
pub extern "C" fn fw_sparkle_active(i: i32) -> i32 {
    game().sparkles.items[i as usize].active as i32
}

#[no_mangle]
pub extern "C" fn fw_sparkle_x(i: i32) -> i32 {
    game().sparkles.items[i as usize].x
}

#[no_mangle]
pub extern "C" fn fw_sparkle_y(i: i32) -> i32 {
    game().sparkles.items[i as usize].y
}

#[no_mangle]
pub extern "C" fn fw_sparkle_frame(i: i32) -> i32 {
    game().sparkles.items[i as usize].anim.frame()
}

#[cfg(target_os = "none")]
extern "C" {
    fn OSReport(format: *const u8, ...);
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let line = info.location().map(|l| l.line()).unwrap_or(0);
    unsafe {
        OSReport(b"RUST PANIC line %u\n\0".as_ptr(), line);
    }
    loop {}
}
