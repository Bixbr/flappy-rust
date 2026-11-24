mod assets;
mod atlas;

use atlas::Atlas;
use flappy_core::game::{
    self, Game, Point, BIRD_SPRITE_OFFSET, EV_DIE, EV_HIT, EV_POINT, EV_SWOOSH, EV_WING,
    MENU_BTN_Y, MENU_PLAY_X, MENU_RATE_X, MENU_RATE_Y, MENU_SCORE_X, PIPE_GAP, PIPE_H,
    PIPE_SPACING, PIPE_W, VH, VW,
};
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

const PLAY_W: f32 = 288.0;
const PLAY_H: f32 = 512.0;
const GEAR_SIZE: f32 = 22.0;
const SETTINGS_PAD: f32 = 8.0;
const SETTINGS_HEADER: f32 = 16.0;
const SCORE_URL: &str = "https://bixed.xyz";

fn open_url(url: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = url;
    }
}

#[derive(Clone, Copy, Default)]
struct Settings {
    widescreen: bool,
    fit_screen: bool,
}

#[derive(Clone, Copy, Default)]
struct UiState {
    settings_open: bool,
}

#[derive(Clone, Copy)]
struct View {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    ppu: f32,
    screen_x: f32,
    screen_y: f32,
}

// ---------------------------------------------------------------------------
// Sounds
// ---------------------------------------------------------------------------
struct Sounds {
    wing: Option<Sound>,
    point: Option<Sound>,
    hit: Option<Sound>,
    die: Option<Sound>,
    swoosh: Option<Sound>,
}

impl Sounds {
    async fn load() -> Self {
        Self {
            wing: load_sound_from_bytes(assets::SFX_WING).await.ok(),
            point: load_sound_from_bytes(assets::SFX_POINT).await.ok(),
            hit: load_sound_from_bytes(assets::SFX_HIT).await.ok(),
            die: load_sound_from_bytes(assets::SFX_DIE).await.ok(),
            swoosh: load_sound_from_bytes(assets::SFX_SWOOSH).await.ok(),
        }
    }

    fn play(&self, sound: &Option<Sound>) {
        if let Some(sound) = sound {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: 1.0,
                },
            );
        }
    }

    fn play_events(&self, events: u16) {
        if events & EV_WING != 0 {
            self.play(&self.wing);
        }
        if events & EV_POINT != 0 {
            self.play(&self.point);
        }
        if events & EV_HIT != 0 {
            self.play(&self.hit);
        }
        if events & EV_DIE != 0 {
            self.play(&self.die);
        }
        if events & EV_SWOOSH != 0 {
            self.play(&self.swoosh);
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------
fn draw_pipe_pair(atlas: &Atlas, x: i32, y: i32) {
    atlas.draw("pipe_up", x as f32, y as f32, 1.0, 0.0, WHITE);
    atlas.draw(
        "pipe_down",
        x as f32,
        (y - PIPE_H - PIPE_GAP) as f32,
        1.0,
        0.0,
        WHITE,
    );
}

fn draw_land(atlas: &Atlas, land_x: i32, view: &View) {
    let tile_w = atlas.width("land");
    if tile_w <= 0.0 {
        return;
    }
    let offset = (land_x as f32 - view.x).rem_euclid(tile_w);
    let mut x = view.x + offset - tile_w;
    while x < view.x + view.w {
        atlas.draw("land", x, (512 - 112) as f32, 1.0, 0.0, WHITE);
        x += tile_w;
    }

    let r = atlas.rect("land");
    if view.y + view.h > 512.0 && r.h > 0.0 {
        draw_texture_ex(
            &atlas.tex,
            view.x,
            512.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(view.w, view.y + view.h - 512.0)),
                source: Some(Rect::new(r.x, r.y + r.h - 1.0, r.w, 1.0)),
                ..Default::default()
            },
        );
    }
}

fn draw_background(atlas: &Atlas, g: &Game, view: &View) {
    let name = if g.bg_night { "bg_night" } else { "bg_day" };
    let r = atlas.rect(name);
    if r.w <= 0.0 || r.h <= 0.0 {
        return;
    }

    let start = (view.x / r.w).floor() * r.w;
    let mut x = start;
    while x < view.x + view.w {
        atlas.draw(name, x, 0.0, 1.0, 0.0, WHITE);
        x += r.w;
    }

    if view.y < 0.0 {
        draw_texture_ex(
            &atlas.tex,
            view.x,
            view.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(view.w, -view.y)),
                source: Some(Rect::new(r.x, r.y, r.w, 1.0)),
                ..Default::default()
            },
        );
    }
}

fn draw_hud(atlas: &Atlas, score: i32) {
    let text = score.to_string();
    let mut total = 2i32;
    for ch in text.chars() {
        let digit = ch.to_digit(10).unwrap() as i32;
        let name = format!("font_{:03}", 48 + digit);
        total += atlas.width(&name) as i32 - 4;
    }
    let mut x = 144 - total / 2;
    let y = 100 - 44 / 2;
    for ch in text.chars() {
        let digit = ch.to_digit(10).unwrap() as i32;
        let name = format!("font_{:03}", 48 + digit);
        let w = atlas.width(&name) as i32;
        atlas.draw(&name, x as f32, y as f32, 1.0, 0.0, WHITE);
        x += w - 4;
    }
}

fn draw_number_score(atlas: &Atlas, value: i32, right_x: i32, y: i32) {
    let mut v = value.max(0);
    let mut x = right_x - 16;
    let mut count = 10;
    let mut leading = true;
    while count > 0 {
        if v > 0 || leading {
            let digit = v % 10;
            let name = format!("number_score_0{}", digit);
            atlas.draw(&name, x as f32, y as f32, 1.0, 0.0, WHITE);
            x -= 16;
            v /= 10;
            leading = false;
        }
        count -= 1;
    }
}

fn draw_panel(atlas: &Atlas, panel: &game::Panel) {
    if !panel.visible {
        return;
    }
    atlas.draw(
        "score_panel",
        panel.x as f32,
        panel.y as f32,
        1.0,
        0.0,
        WHITE,
    );
    draw_number_score(atlas, panel.display, panel.x + 210, panel.y + 36);
    draw_number_score(atlas, panel.best, panel.x + 210, panel.y + 78);
    if panel.new_badge {
        atlas.draw(
            "new",
            (panel.x + 142) as f32,
            (panel.y + 60) as f32,
            1.0,
            0.0,
            WHITE,
        );
    }
    if let Some(medal) = panel.medal {
        let name = format!("medals_{}", medal);
        atlas.draw(
            &name,
            panel.medal_x as f32,
            panel.medal_y as f32,
            1.0,
            0.0,
            WHITE,
        );
    }
}

fn draw_sparkles(atlas: &Atlas, sparkles: &game::Sparkles) {
    for s in &sparkles.items {
        if s.active && !s.anim.done {
            let name = format!("blink_0{}", s.anim.frame());
            atlas.draw(&name, (s.x - 4) as f32, (s.y - 4) as f32, 1.0, 0.0, WHITE);
        }
    }
}

fn menu_like(g: &Game) -> bool {
    g.menu || (g.panel.visible && g.panel.state == 2)
}

fn draw_gear(x: f32, y: f32, highlight: bool) {
    let cx = x + GEAR_SIZE * 0.5;
    let cy = y + GEAR_SIZE * 0.5;
    let body = Color::new(0.32, 0.19, 0.09, 1.0);
    let face = if highlight {
        Color::new(1.0, 0.96, 0.78, 1.0)
    } else {
        Color::new(0.98, 0.90, 0.66, 1.0)
    };
    draw_rectangle(
        x - 1.0,
        y + 1.0,
        GEAR_SIZE + 4.0,
        GEAR_SIZE + 4.0,
        Color::new(0.0, 0.0, 0.0, 0.18),
    );
    draw_rectangle(x - 2.0, y - 2.0, GEAR_SIZE + 4.0, GEAR_SIZE + 4.0, face);
    draw_rectangle_lines(
        x - 2.0,
        y - 2.0,
        GEAR_SIZE + 4.0,
        GEAR_SIZE + 4.0,
        1.0,
        body,
    );
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let (s, c) = a.sin_cos();
        draw_line(
            cx + c * 4.0,
            cy + s * 4.0,
            cx + c * 8.0,
            cy + s * 8.0,
            2.0,
            body,
        );
    }
    draw_circle(cx, cy, 5.0, body);
    draw_circle(cx, cy, 2.0, face);
}

fn draw_check(x: f32, y: f32, on: bool) {
    let body = Color::new(0.32, 0.19, 0.09, 1.0);
    draw_rectangle(x, y, 10.0, 10.0, Color::new(0.98, 0.93, 0.78, 1.0));
    draw_rectangle_lines(x, y, 10.0, 10.0, 1.0, body);
    if on {
        draw_line(x + 2.0, y + 5.0, x + 4.5, y + 7.5, 1.5, body);
        draw_line(x + 4.5, y + 7.5, x + 8.0, y + 2.5, 1.5, body);
    }
}

fn settings_rows(settings: &Settings) -> Vec<(&'static str, bool)> {
    let mut rows = vec![("WIDESCREEN", settings.widescreen)];
    if cfg!(target_os = "android") {
        rows.push(("FIT TO SCREEN", settings.fit_screen));
    }
    rows
}

fn gear_rect(view: &View) -> Rect {
    Rect::new(
        view.x + view.w - GEAR_SIZE - 6.0,
        view.y + 6.0,
        GEAR_SIZE,
        GEAR_SIZE,
    )
}

fn settings_panel_rect(view: &View, settings: &Settings) -> Rect {
    let rows = settings_rows(settings).len() as f32;
    let h = SETTINGS_HEADER + SETTINGS_PAD * 2.0 + rows * 18.0;
    let w = 176.0;
    let gear = gear_rect(view);
    Rect::new(gear.x + GEAR_SIZE - w, gear.y + GEAR_SIZE + 6.0, w, h)
}

fn settings_row_y(panel: Rect, index: usize) -> f32 {
    panel.y + SETTINGS_HEADER + SETTINGS_PAD + index as f32 * 18.0
}

fn draw_settings(g: &Game, settings: &Settings, ui: &UiState, view: &View) {
    if !menu_like(g) {
        return;
    }
    let pos = screen_to_virtual(Vec2::new(mouse_position().0, mouse_position().1), view);
    let gear = gear_rect(view);
    let gear_hit = Rect::new(gear.x - 2.0, gear.y - 2.0, gear.w + 4.0, gear.h + 4.0);
    draw_gear(gear.x, gear.y, point_in(pos, gear_hit));
    if !ui.settings_open {
        return;
    }

    let panel = settings_panel_rect(view, settings);
    let body = Color::new(0.32, 0.19, 0.09, 1.0);
    let face = Color::new(0.98, 0.90, 0.66, 1.0);
    let hover = Color::new(1.0, 0.95, 0.78, 1.0);
    draw_rectangle(
        panel.x + 2.0,
        panel.y + 2.0,
        panel.w,
        panel.h,
        Color::new(0.0, 0.0, 0.0, 0.22),
    );
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, face);
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, body);
    draw_text(
        "SETTINGS",
        panel.x + SETTINGS_PAD,
        panel.y + 12.0,
        14.0,
        body,
    );
    draw_line(
        panel.x + SETTINGS_PAD,
        panel.y + SETTINGS_HEADER,
        panel.x + panel.w - SETTINGS_PAD,
        panel.y + SETTINGS_HEADER,
        1.0,
        body,
    );

    for (i, (label, on)) in settings_rows(settings).iter().enumerate() {
        let y = settings_row_y(panel, i);
        let row = Rect::new(panel.x + 2.0, y - 3.0, panel.w - 4.0, 16.0);
        if point_in(pos, row) {
            draw_rectangle(row.x, row.y, row.w, row.h, hover);
        }
        draw_check(panel.x + SETTINGS_PAD, y, *on);
        draw_text(label, panel.x + SETTINGS_PAD + 16.0, y + 9.0, 14.0, body);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_game(
    atlas: &Atlas,
    g: &Game,
    settings: &Settings,
    ui: &UiState,
    view: &View,
    ghosts: &[(f32, i32)],
    previews: &[(i32, i32)],
) {
    draw_background(atlas, g, view);

    if !g.menu && g.pipes_pending <= 0 {
        if settings.widescreen {
            for &(x, y) in ghosts {
                draw_pipe_pair(atlas, x as i32, y);
            }
            for i in 0..3 {
                let x = g.pipes[i].0;
                if x as f32 + PIPE_W as f32 >= view.x && (x as f32) <= view.x + view.w {
                    draw_pipe_pair(atlas, x, g.pipes[i].1);
                }
            }
            for &(x, height) in previews {
                if x as f32 > view.x + view.w {
                    break;
                }
                if x as f32 + PIPE_W as f32 >= view.x {
                    draw_pipe_pair(atlas, x, height);
                }
            }
        } else {
            for i in 0..2 {
                draw_pipe_pair(atlas, g.pipes[i].0, g.pipes[i].1);
            }
            if g.pipes[2].0 < 288 {
                draw_pipe_pair(atlas, g.pipes[2].0, g.pipes[2].1);
            }
        }
    }

    if g.menu {
        let title_x = (288.0 - atlas.width("title")) / 2.0;
        atlas.draw("title", title_x, 150.0, 1.0, 0.0, WHITE);

        let x = 120 - BIRD_SPRITE_OFFSET;
        let y = 218 - BIRD_SPRITE_OFFSET;
        let name = format!("bird{}_{}", g.bird.variant, g.bird.frame_index());
        atlas.draw(&name, x as f32, y as f32 + g.bird.bob_y, 1.0, 0.0, WHITE);

        draw_land(atlas, g.land_x, view);
    } else {
        if g.over_text.visible {
            let x = (288.0 - atlas.width("text_game_over")) / 2.0;
            atlas.draw(
                "text_game_over",
                x,
                (g.over_text.offset + 130) as f32,
                1.0,
                0.0,
                Color::new(1.0, 1.0, 1.0, g.over_text.alpha.value),
            );
        } else {
            draw_hud(atlas, g.score);
        }

        let name = format!("bird{}_{}", g.bird.variant, g.bird.frame_index());
        let x = (g.bird.x - BIRD_SPRITE_OFFSET) as f32;
        let y = (g.bird.y - BIRD_SPRITE_OFFSET) as f32 + g.bird.bob_y;
        if g.bird.rot != 0.0 {
            atlas.draw_smooth(&name, x, y, 1.0, g.bird.rot, WHITE);
        } else {
            atlas.draw(&name, x, y, 1.0, 0.0, WHITE);
        }

        draw_land(atlas, g.land_x, view);
    }

    if g.ready.visible {
        let alpha = g.ready.alpha.value;
        let color = Color::new(1.0, 1.0, 1.0, alpha);
        let x = (288.0 - atlas.width("text_ready")) / 2.0;
        atlas.draw("text_ready", x, 146.0, 1.0, 0.0, color);
        let x = (288.0 - atlas.width("tutorial")) / 2.0;
        atlas.draw("tutorial", x, 220.0, 1.0, 0.0, color);
    }

    if g.menu {
        let x = (288.0 - atlas.width("brand_copyright")) / 2.0;
        atlas.draw("brand_copyright", x, 432.0 - 14.0, 1.0, 0.0, WHITE);
    }

    if g.play_visible {
        atlas.draw(
            "button_play",
            MENU_PLAY_X as f32,
            MENU_BTN_Y as f32,
            1.0,
            0.0,
            WHITE,
        );
    }
    if g.score_visible {
        atlas.draw(
            "button_score",
            MENU_SCORE_X as f32,
            MENU_BTN_Y as f32,
            1.0,
            0.0,
            WHITE,
        );
    }
    if g.rate_visible {
        atlas.draw(
            "button_rate",
            MENU_RATE_X as f32,
            MENU_RATE_Y as f32,
            1.0,
            0.0,
            WHITE,
        );
    }

    draw_panel(atlas, &g.panel);
    draw_sparkles(atlas, &g.sparkles);
    draw_settings(g, settings, ui, view);

    if !g.fade.done || g.fade.value != 0.0 {
        draw_rectangle(
            view.x,
            view.y,
            view.w,
            view.h,
            Color::new(0.0, 0.0, 0.0, g.fade.value.clamp(0.0, 1.0)),
        );
    }
    if !g.flash.done || g.flash.value != 0.0 {
        draw_rectangle(
            view.x,
            view.y,
            view.w,
            view.h,
            Color::new(1.0, 1.0, 1.0, g.flash.value.clamp(0.0, 1.0)),
        );
    }
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------
fn compute_view(settings: &Settings) -> View {
    let sw = screen_width();
    let sh = screen_height();
    let fit_w = sw / PLAY_W;
    let fit_h = sh / PLAY_H;
    let contain = fit_w.min(fit_h);

    let (ppu, vis_w, vis_h) = if settings.widescreen && settings.fit_screen {
        (contain, sw / contain, sh / contain)
    } else if settings.widescreen {
        if fit_w > fit_h {
            (fit_h, sw / fit_h, PLAY_H)
        } else {
            (contain, PLAY_W, PLAY_H)
        }
    } else if settings.fit_screen {
        if fit_h > fit_w {
            (fit_w, PLAY_W, sh / fit_w)
        } else {
            (contain, PLAY_W, PLAY_H)
        }
    } else {
        (contain, PLAY_W, PLAY_H)
    };

    let x = (PLAY_W - vis_w) * 0.5;
    let y = (PLAY_H - vis_h) * 0.5;
    let dest_w = vis_w * ppu;
    let dest_h = vis_h * ppu;
    View {
        x,
        y,
        w: vis_w,
        h: vis_h,
        ppu,
        screen_x: (sw - dest_w) * 0.5,
        screen_y: (sh - dest_h) * 0.5,
    }
}

fn screen_to_virtual(p: Vec2, view: &View) -> Vec2 {
    Vec2::new(
        view.x + (p.x - view.screen_x) / view.ppu,
        view.y + (p.y - view.screen_y) / view.ppu,
    )
}

fn point_in(p: Vec2, r: Rect) -> bool {
    p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h
}

#[allow(clippy::too_many_arguments)]
fn handle_ui(
    settings: &mut Settings,
    ui: &mut UiState,
    tap: &mut bool,
    release: &mut bool,
    pos: Vec2,
    active: bool,
    view: &View,
    score_visible: bool,
) {
    if !active {
        ui.settings_open = false;
        return;
    }

    let gear = gear_rect(view);
    let gear = Rect::new(gear.x - 2.0, gear.y - 2.0, gear.w + 4.0, gear.h + 4.0);
    if *tap && point_in(pos, gear) {
        ui.settings_open = !ui.settings_open;
        *tap = false;
        *release = false;
        return;
    }

    if !ui.settings_open {
        let score_button = Rect::new(MENU_SCORE_X as f32, MENU_BTN_Y as f32, 116.0, 70.0);
        if score_visible && *tap && point_in(pos, score_button) {
            open_url(SCORE_URL);
            *tap = false;
            *release = false;
        }
        return;
    }

    let panel = settings_panel_rect(view, settings);
    let rows = settings_rows(settings).len();
    if *tap {
        for i in 0..rows {
            let y = settings_row_y(panel, i);
            let row = Rect::new(panel.x + 2.0, y - 3.0, panel.w - 4.0, 16.0);
            if point_in(pos, row) {
                if i == 0 {
                    settings.widescreen = !settings.widescreen;
                } else {
                    settings.fit_screen = !settings.fit_screen;
                    #[cfg(target_os = "android")]
                    macroquad::window::set_fullscreen(settings.fit_screen);
                }
                *tap = false;
                *release = false;
                return;
            }
        }
        ui.settings_open = false;
        *tap = false;
        *release = false;
        return;
    }
    if *release && (point_in(pos, panel) || point_in(pos, gear)) {
        *release = false;
    }
}

struct FrameInput {
    tap: bool,
    release: bool,
    pos: Vec2,
}

fn read_input(view: &View) -> FrameInput {
    let mut tap = false;
    let mut release = false;
    let mut pos = screen_to_virtual(Vec2::new(mouse_position().0, mouse_position().1), view);

    if is_mouse_button_pressed(MouseButton::Left) {
        tap = true;
    }
    if is_mouse_button_released(MouseButton::Left) {
        release = true;
    }

    for touch in touches() {
        match touch.phase {
            TouchPhase::Started => {
                tap = true;
                pos = screen_to_virtual(touch.position, view);
            }
            TouchPhase::Ended => {
                release = true;
                pos = screen_to_virtual(touch.position, view);
            }
            _ => {}
        }
    }

    if is_key_pressed(KeyCode::Space) {
        tap = true;
    }

    FrameInput { tap, release, pos }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Bird (Rust port)".to_owned(),
        window_width: 432,
        window_height: 768,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let atlas = Atlas::load().await;
    let sounds = Sounds::load().await;

    let seed = (get_time() * 1000.0) as i32;
    let mut game = Game::new(seed);
    let mut settings = Settings::default();
    let mut ui = UiState::default();

    let mut target = render_target(VW, VH);
    target.texture.set_filter(FilterMode::Nearest);
    let mut target_size = (VW, VH);
    let mut ghosts: Vec<(f32, i32)> = Vec::new();
    let mut previews: Vec<(i32, i32)> = Vec::new();

    let mut accum = 0.0f32;

    loop {
        let view = compute_view(&settings);
        let input = read_input(&view);
        let mut tap = input.tap;
        let mut release = input.release;
        handle_ui(
            &mut settings,
            &mut ui,
            &mut tap,
            &mut release,
            input.pos,
            menu_like(&game),
            &view,
            game.score_visible,
        );
        game.handle_input(tap, release, Point::new(input.pos.x, input.pos.y));

        accum += get_frame_time().min(0.1);
        let mut steps = 0;
        while accum >= game::FIXED_DT && steps < 5 {
            let before = game.pipes[0];
            let was_running = !game.menu && game.pipes_pending <= 0;
            game.tick();

            if settings.widescreen {
                if game.menu || game.pipes_pending > 0 {
                    ghosts.clear();
                    previews.clear();
                }
                let speed = game.pipe_speed as f32;
                for ghost in &mut ghosts {
                    ghost.0 -= speed;
                }
                if was_running && game.pipes[0].0 > before.0 {
                    ghosts.push((before.0 as f32 - speed, before.1));
                }
                ghosts.retain(|g| g.0 + PIPE_W as f32 >= view.x - 1.0);

                if game.pipe_speed > 0 && !game.menu && game.pipes_pending <= 0 {
                    let step = PIPE_SPACING + PIPE_W;
                    let mut x = game.pipes[2].0 + step;
                    previews.clear();
                    for height in game.pipe_previews() {
                        previews.push((x, height));
                        x += step;
                    }
                }
            } else {
                ghosts.clear();
                previews.clear();
            }

            sounds.play_events(game.take_events());
            accum -= game::FIXED_DT;
            steps += 1;
        }
        if steps == 5 {
            accum = 0.0;
        }

        let tw = (view.w * view.ppu).ceil().max(1.0) as u32;
        let th = (view.h * view.ppu).ceil().max(1.0) as u32;
        if target_size != (tw, th) {
            target = render_target(tw, th);
            target.texture.set_filter(FilterMode::Nearest);
            target_size = (tw, th);
        }

        let mut camera = Camera2D::from_display_rect(Rect::new(view.x, view.y, view.w, view.h));
        camera.render_target = Some(target.clone());

        set_camera(&camera);
        clear_background(BLACK);
        draw_game(&atlas, &game, &settings, &ui, &view, &ghosts, &previews);

        set_default_camera();
        clear_background(BLACK);
        let sx = ((screen_width() - tw as f32) * 0.5).floor();
        let sy = ((screen_height() - th as f32) * 0.5).floor();
        draw_texture_ex(
            &target.texture,
            sx,
            sy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(tw as f32, th as f32)),
                flip_y: true,
                ..Default::default()
            },
        );

        next_frame().await;
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn quad_main() {
    main();
}
