#![no_std]
#![no_main]

use core::ffi::{c_int, c_void};
use core::mem::zeroed;
use core::panic::PanicInfo;

use flappy_core::game::{
    Game, Point, BIRD_SPRITE_OFFSET, MENU_BTN_Y, MENU_PLAY_X, MENU_RATE_X, MENU_RATE_Y,
    MENU_SCORE_X, PIPE_GAP, PIPE_H,
};
use flappy_core::{AtlasMap, SpriteRect};

const ATLAS_PNG: &[u8] = include_bytes!("../../../assets/atlas.png");
const ATLAS_TXT: &str = include_str!("../../../assets/atlas.txt");

const SCREEN_W: f32 = 960.0;
const SCREEN_H: f32 = 544.0;
const SCALE: f32 = SCREEN_H / 512.0;
const OFFSET_X: f32 = (SCREEN_W - 288.0 * SCALE) * 0.5;
const DEG_TO_RAD: f32 = 0.017453293;

const SCE_CTRL_SELECT: u32 = 0x0000_0001;
const SCE_CTRL_START: u32 = 0x0000_0008;
const SCE_CTRL_UP: u32 = 0x0000_0010;
const SCE_CTRL_RTRIGGER: u32 = 0x0000_0200;
const SCE_CTRL_CIRCLE: u32 = 0x0000_2000;
const SCE_CTRL_CROSS: u32 = 0x0000_4000;

const BIRD_NAMES: [[&str; 3]; 3] = [
    ["bird0_0", "bird0_1", "bird0_2"],
    ["bird1_0", "bird1_1", "bird1_2"],
    ["bird2_0", "bird2_1", "bird2_2"],
];
const FONT_NAMES: [&str; 10] = [
    "font_048", "font_049", "font_050", "font_051", "font_052", "font_053", "font_054", "font_055",
    "font_056", "font_057",
];
const NUMBER_NAMES: [&str; 10] = [
    "number_score_00",
    "number_score_01",
    "number_score_02",
    "number_score_03",
    "number_score_04",
    "number_score_05",
    "number_score_06",
    "number_score_07",
    "number_score_08",
    "number_score_09",
];
const BLINK_NAMES: [&str; 3] = ["blink_00", "blink_01", "blink_02"];
const MEDAL_NAMES: [&str; 4] = ["medals_0", "medals_1", "medals_2", "medals_3"];

#[repr(C)]
struct SceCtrlData {
    timestamp: u64,
    buttons: u32,
    lx: u8,
    ly: u8,
    rx: u8,
    ry: u8,
    up: u8,
    right: u8,
    down: u8,
    left: u8,
    lt: u8,
    rt: u8,
    l1: u8,
    r1: u8,
    triangle: u8,
    circle: u8,
    cross: u8,
    square: u8,
    reserved: [u8; 4],
}

#[repr(C)]
struct SceTouchReport {
    id: u8,
    force: u8,
    x: i16,
    y: i16,
    reserved: [u8; 8],
    info: u16,
}

#[repr(C)]
struct SceTouchData {
    timestamp: u64,
    status: u32,
    report_num: u32,
    report: [SceTouchReport; 8],
}

extern "C" {
    fn vita2d_init() -> c_int;
    fn vita2d_fini();
    fn vita2d_clear_screen();
    fn vita2d_start_drawing();
    fn vita2d_end_drawing();
    fn vita2d_swap_buffers();
    fn vita2d_set_clear_color(color: u32);
    fn vita2d_enable_clipping();
    fn vita2d_set_clip_rectangle(x_min: c_int, y_min: c_int, x_max: c_int, y_max: c_int);
    fn vita2d_draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn vita2d_load_PNG_buffer(buffer: *const c_void) -> *mut c_void;
    fn vita2d_texture_set_filters(texture: *mut c_void, min: c_int, mag: c_int);
    fn vita2d_draw_texture_tint_part_scale(
        texture: *const c_void,
        x: f32,
        y: f32,
        tex_x: f32,
        tex_y: f32,
        tex_w: f32,
        tex_h: f32,
        x_scale: f32,
        y_scale: f32,
        color: u32,
    );
    #[allow(clippy::too_many_arguments)]
    fn vita2d_draw_texture_part_tint_scale_rotate(
        texture: *const c_void,
        x: f32,
        y: f32,
        tex_x: f32,
        tex_y: f32,
        tex_w: f32,
        tex_h: f32,
        x_scale: f32,
        y_scale: f32,
        rad: f32,
        color: u32,
    );
    fn sceCtrlSetSamplingMode(mode: c_int) -> c_int;
    fn sceCtrlPeekBufferPositive(port: c_int, data: *mut SceCtrlData, count: c_int) -> c_int;
    fn sceTouchPeek(port: u32, data: *mut SceTouchData, count: u32) -> c_int;
    fn sceKernelPowerTick(kind: c_int) -> c_int;
    fn sceKernelExitProcess(res: c_int);
    fn sceDisplayWaitVblankStart() -> c_int;
}

const fn rgba(r: u32, g: u32, b: u32, a: u32) -> u32 {
    (a << 24) | (b << 16) | (g << 8) | r
}

fn screen_to_virtual(x: f32, y: f32) -> Point {
    Point::new((x - OFFSET_X) / SCALE, y / SCALE)
}

unsafe fn draw_sprite(
    tex: *const c_void,
    atlas: &AtlasMap,
    name: &str,
    x: f32,
    y: f32,
    rotation: f32,
    alpha: f32,
) {
    let r: SpriteRect = atlas.rect(name);
    if r.w == 0.0 {
        return;
    }
    let color = rgba(255, 255, 255, (alpha.clamp(0.0, 1.0) * 255.0) as u32);
    let sx = OFFSET_X + x * SCALE;
    let sy = y * SCALE;
    if rotation != 0.0 {
        vita2d_draw_texture_part_tint_scale_rotate(
            tex,
            sx + r.w * SCALE * 0.5,
            sy + r.h * SCALE * 0.5,
            r.x,
            r.y,
            r.w,
            r.h,
            SCALE,
            SCALE,
            rotation * DEG_TO_RAD,
            color,
        );
    } else {
        vita2d_draw_texture_tint_part_scale(tex, sx, sy, r.x, r.y, r.w, r.h, SCALE, SCALE, color);
    }
}

unsafe fn draw_pipe_pair(tex: *const c_void, atlas: &AtlasMap, x: i32, y: i32) {
    draw_sprite(tex, atlas, "pipe_up", x as f32, y as f32, 0.0, 1.0);
    draw_sprite(
        tex,
        atlas,
        "pipe_down",
        x as f32,
        (y - PIPE_H - PIPE_GAP) as f32,
        0.0,
        1.0,
    );
}

unsafe fn draw_hud(tex: *const c_void, atlas: &AtlasMap, score: i32) {
    let mut digits = [0i32; 10];
    let mut n = 0usize;
    let mut v = if score < 0 { 0 } else { score };
    if v == 0 {
        digits[0] = 0;
        n = 1;
    } else {
        while v > 0 && n < digits.len() {
            digits[n] = v % 10;
            v /= 10;
            n += 1;
        }
    }

    let mut total = 2i32;
    for i in (0..n).rev() {
        total += atlas.rect(FONT_NAMES[digits[i] as usize]).w as i32 - 4;
    }
    let mut x = 144 - total / 2;
    let y = 100 - 44 / 2;
    for i in (0..n).rev() {
        let name = FONT_NAMES[digits[i] as usize];
        let w = atlas.rect(name).w as i32;
        draw_sprite(tex, atlas, name, x as f32, y as f32, 0.0, 1.0);
        x += w - 4;
    }
}

unsafe fn draw_number(tex: *const c_void, atlas: &AtlasMap, value: i32, right_x: i32, y: i32) {
    let mut v = if value < 0 { 0 } else { value };
    let mut x = right_x - 16;
    let mut count = 10;
    let mut leading = true;
    while count > 0 {
        if v > 0 || leading {
            let digit = v % 10;
            draw_sprite(
                tex,
                atlas,
                NUMBER_NAMES[digit as usize],
                x as f32,
                y as f32,
                0.0,
                1.0,
            );
            x -= 16;
            v /= 10;
            leading = false;
        }
        count -= 1;
    }
}

unsafe fn draw_scene(g: &Game, tex: *const c_void, atlas: &AtlasMap) {
    let bg = if g.bg_night { "bg_night" } else { "bg_day" };
    draw_sprite(tex, atlas, bg, 0.0, 0.0, 0.0, 1.0);

    if !g.menu && g.pipes_pending <= 0 {
        for i in 0..2 {
            draw_pipe_pair(tex, atlas, g.pipes[i].0, g.pipes[i].1);
        }
        if g.pipes[2].0 < 288 {
            draw_pipe_pair(tex, atlas, g.pipes[2].0, g.pipes[2].1);
        }
    }

    if g.menu {
        let title_w = atlas.rect("title").w;
        draw_sprite(
            tex,
            atlas,
            "title",
            (288.0 - title_w) / 2.0,
            150.0,
            0.0,
            1.0,
        );

        let name = BIRD_NAMES[g.bird.variant as usize][g.bird.frame_index() as usize];
        draw_sprite(
            tex,
            atlas,
            name,
            (120 - BIRD_SPRITE_OFFSET) as f32,
            (218 - BIRD_SPRITE_OFFSET) as f32 + g.bird.bob_y,
            0.0,
            1.0,
        );
        draw_sprite(tex, atlas, "land", g.land_x as f32, 400.0, 0.0, 1.0);
    } else {
        if g.over_text.visible {
            let w = atlas.rect("text_game_over").w;
            draw_sprite(
                tex,
                atlas,
                "text_game_over",
                (288.0 - w) / 2.0,
                (g.over_text.offset + 130) as f32,
                0.0,
                g.over_text.alpha.value,
            );
        } else {
            draw_hud(tex, atlas, g.score);
        }

        let name = BIRD_NAMES[g.bird.variant as usize][g.bird.frame_index() as usize];
        draw_sprite(
            tex,
            atlas,
            name,
            (g.bird.x - BIRD_SPRITE_OFFSET) as f32,
            (g.bird.y - BIRD_SPRITE_OFFSET) as f32 + g.bird.bob_y,
            g.bird.rot,
            1.0,
        );
        draw_sprite(tex, atlas, "land", g.land_x as f32, 400.0, 0.0, 1.0);
    }

    if g.ready.visible {
        let alpha = g.ready.alpha.value;
        let w = atlas.rect("text_ready").w;
        draw_sprite(
            tex,
            atlas,
            "text_ready",
            (288.0 - w) / 2.0,
            146.0,
            0.0,
            alpha,
        );
        let w = atlas.rect("tutorial").w;
        draw_sprite(tex, atlas, "tutorial", (288.0 - w) / 2.0, 220.0, 0.0, alpha);
    }

    if g.menu {
        let w = atlas.rect("brand_copyright").w;
        draw_sprite(
            tex,
            atlas,
            "brand_copyright",
            (288.0 - w) / 2.0,
            418.0,
            0.0,
            1.0,
        );
    }

    if g.play_visible {
        draw_sprite(
            tex,
            atlas,
            "button_play",
            MENU_PLAY_X as f32,
            MENU_BTN_Y as f32,
            0.0,
            1.0,
        );
    }
    if g.score_visible {
        draw_sprite(
            tex,
            atlas,
            "button_score",
            MENU_SCORE_X as f32,
            MENU_BTN_Y as f32,
            0.0,
            1.0,
        );
    }
    if g.rate_visible {
        draw_sprite(
            tex,
            atlas,
            "button_rate",
            MENU_RATE_X as f32,
            MENU_RATE_Y as f32,
            0.0,
            1.0,
        );
    }

    if g.panel.visible {
        let p = &g.panel;
        draw_sprite(tex, atlas, "score_panel", p.x as f32, p.y as f32, 0.0, 1.0);
        draw_number(tex, atlas, p.display, p.x + 210, p.y + 36);
        draw_number(tex, atlas, p.best, p.x + 210, p.y + 78);
        if p.new_badge {
            draw_sprite(
                tex,
                atlas,
                "new",
                (p.x + 142) as f32,
                (p.y + 60) as f32,
                0.0,
                1.0,
            );
        }
        if let Some(medal) = p.medal {
            draw_sprite(
                tex,
                atlas,
                MEDAL_NAMES[medal],
                p.medal_x as f32,
                p.medal_y as f32,
                0.0,
                1.0,
            );
        }
    }

    for s in &g.sparkles.items {
        if s.active && !s.anim.done {
            let idx = s.anim.frame().clamp(0, 2) as usize;
            draw_sprite(
                tex,
                atlas,
                BLINK_NAMES[idx],
                (s.x - 4) as f32,
                (s.y - 4) as f32,
                0.0,
                1.0,
            );
        }
    }

    if !g.fade.done || g.fade.value != 0.0 {
        vita2d_draw_rectangle(
            OFFSET_X,
            0.0,
            288.0 * SCALE,
            512.0 * SCALE,
            rgba(0, 0, 0, (g.fade.value.clamp(0.0, 1.0) * 255.0) as u32),
        );
    }
    if !g.flash.done || g.flash.value != 0.0 {
        vita2d_draw_rectangle(
            OFFSET_X,
            0.0,
            288.0 * SCALE,
            512.0 * SCALE,
            rgba(
                255,
                255,
                255,
                (g.flash.value.clamp(0.0, 1.0) * 255.0) as u32,
            ),
        );
    }
}

#[no_mangle]
pub extern "C" fn main() -> c_int {
    unsafe {
        vita2d_init();
        vita2d_set_clear_color(rgba(0, 0, 0, 255));
        let tex = vita2d_load_PNG_buffer(ATLAS_PNG.as_ptr() as *const c_void);
        if tex.is_null() {
            sceKernelExitProcess(1);
            loop {}
        }
        vita2d_texture_set_filters(tex, 0, 0);
        let atlas = AtlasMap::parse(ATLAS_TXT);
        sceCtrlSetSamplingMode(1);

        let seed = (sceKernelGetProcessTimeLow() & 0x7fff_ffff) as i32;
        let mut game = Game::new(seed);
        let mut prev_buttons = 0u32;
        let mut touch_was_down = false;
        let mut prev_touch = Point::new(0.0, 0.0);
        let mut release = false;

        loop {
            let mut pad: SceCtrlData = zeroed();
            if sceCtrlPeekBufferPositive(0, &mut pad, 1) > 0 {
                let pressed = pad.buttons & !prev_buttons;
                prev_buttons = pad.buttons;
                if pressed & (SCE_CTRL_CROSS | SCE_CTRL_START | SCE_CTRL_UP | SCE_CTRL_RTRIGGER)
                    != 0
                {
                    if game.menu || (game.panel.visible && game.panel.state == 2) {
                        game.activate_play();
                    } else {
                        game.handle_input(true, false, Point::new(0.0, 0.0));
                    }
                }
                if pressed & SCE_CTRL_CIRCLE != 0
                    || (pad.buttons & SCE_CTRL_SELECT != 0 && pad.buttons & SCE_CTRL_RTRIGGER != 0)
                {
                }
            }
            let mut touch: SceTouchData = zeroed();
            let touching = sceTouchPeek(0, &mut touch, 1) > 0 && touch.report_num > 0;
            if touching {
                let r = &touch.report[0];
                prev_touch = screen_to_virtual(r.x as f32 * 0.5, r.y as f32 * 0.5);
                if !touch_was_down {
                    if game.menu || (game.panel.visible && game.panel.state == 2) {
                        game.activate_play();
                    } else {
                        game.handle_input(true, false, prev_touch);
                    }
                }
            } else if touch_was_down {
                release = true;
            }
            touch_was_down = touching;
            if release {
                game.handle_input(false, true, prev_touch);
                release = false;
            }

            let events = game.take_events();
            let _ = events;
            game.tick();

            vita2d_start_drawing();
            vita2d_clear_screen();
            vita2d_enable_clipping();
            vita2d_set_clip_rectangle(
                OFFSET_X as c_int,
                0,
                (OFFSET_X + 288.0 * SCALE) as c_int,
                (512.0 * SCALE) as c_int,
            );
            draw_scene(&game, tex as *const c_void, &atlas);
            vita2d_end_drawing();
            vita2d_swap_buffers();
            sceDisplayWaitVblankStart();
            sceKernelPowerTick(0);
        }
    }
}

extern "C" {
    fn sceKernelGetProcessTimeLow() -> u64;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        sceKernelExitProcess(1);
    }
    loop {}
}
