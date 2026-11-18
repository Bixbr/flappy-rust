mod assets;
mod atlas;

use atlas::Atlas;
use flappy_core::game::{
    self, Game, Point, BIRD_SPRITE_OFFSET, EV_DIE, EV_HIT, EV_POINT, EV_SWOOSH, EV_WING,
    MENU_BTN_Y, MENU_PLAY_X, MENU_RATE_X, MENU_RATE_Y, MENU_SCORE_X, PIPE_GAP, PIPE_H, VHF, VWF,
    VW, VH,
};
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

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

fn draw_land(atlas: &Atlas, land_x: i32) {
    atlas.draw("land", land_x as f32, (512 - 112) as f32, 1.0, 0.0, WHITE);
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
    atlas.draw("score_panel", panel.x as f32, panel.y as f32, 1.0, 0.0, WHITE);
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

fn draw_game(atlas: &Atlas, g: &Game) {
    let bg = if g.bg_night { "bg_night" } else { "bg_day" };
    atlas.draw(bg, 0.0, 0.0, 1.0, 0.0, WHITE);

    if !g.menu && g.pipes_pending <= 0 {
        for i in 0..2 {
            draw_pipe_pair(atlas, g.pipes[i].0, g.pipes[i].1);
        }
        if g.pipes[2].0 < 288 {
            draw_pipe_pair(atlas, g.pipes[2].0, g.pipes[2].1);
        }
    }

    if g.menu {
        let title_x = (288.0 - atlas.width("title")) / 2.0;
        atlas.draw("title", title_x, 150.0, 1.0, 0.0, WHITE);

        let x = 120 - BIRD_SPRITE_OFFSET;
        let y = 218 - BIRD_SPRITE_OFFSET;
        let name = format!("bird{}_{}", g.bird.variant, g.bird.frame_index());
        atlas.draw(
            &name,
            x as f32,
            y as f32 + g.bird.bob_y,
            1.0,
            0.0,
            WHITE,
        );

        draw_land(atlas, g.land_x);
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
        atlas.draw(
            &name,
            (g.bird.x - BIRD_SPRITE_OFFSET) as f32,
            (g.bird.y - BIRD_SPRITE_OFFSET) as f32 + g.bird.bob_y,
            1.0,
            g.bird.rot,
            WHITE,
        );

        draw_land(atlas, g.land_x);
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

    if !g.fade.done || g.fade.value != 0.0 {
        draw_rectangle(
            0.0,
            0.0,
            VWF,
            VHF,
            Color::new(0.0, 0.0, 0.0, g.fade.value.clamp(0.0, 1.0)),
        );
    }
    if !g.flash.done || g.flash.value != 0.0 {
        draw_rectangle(
            0.0,
            0.0,
            VWF,
            VHF,
            Color::new(1.0, 1.0, 1.0, g.flash.value.clamp(0.0, 1.0)),
        );
    }
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------
fn screen_to_virtual(p: Vec2) -> Vec2 {
    let scale = (screen_width() / VWF).min(screen_height() / VHF);
    let ox = (screen_width() - VWF * scale) * 0.5;
    let oy = (screen_height() - VHF * scale) * 0.5;
    Vec2::new((p.x - ox) / scale, (p.y - oy) / scale)
}

struct FrameInput {
    tap: bool,
    release: bool,
    pos: Vec2,
}

fn read_input() -> FrameInput {
    let mut tap = false;
    let mut release = false;
    let mut pos = screen_to_virtual(Vec2::new(mouse_position().0, mouse_position().1));

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
                pos = screen_to_virtual(touch.position);
            }
            TouchPhase::Ended => {
                release = true;
                pos = screen_to_virtual(touch.position);
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

    let target = render_target(VW, VH);
    target.texture.set_filter(FilterMode::Nearest);
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VWF, VHF));
    camera.render_target = Some(target.clone());

    let mut accum = 0.0f32;

    loop {
        let input = read_input();
        game.handle_input(
            input.tap,
            input.release,
            Point::new(input.pos.x, input.pos.y),
        );

        accum += get_frame_time().min(0.1);
        let mut steps = 0;
        while accum >= game::FIXED_DT && steps < 5 {
            game.tick();
            sounds.play_events(game.take_events());
            accum -= game::FIXED_DT;
            steps += 1;
        }
        if steps == 5 {
            accum = 0.0;
        }

        set_camera(&camera);
        clear_background(BLACK);
        draw_game(&atlas, &game);

        set_default_camera();
        clear_background(BLACK);

        let scale = (screen_width() / VWF).min(screen_height() / VHF);
        let w = VWF * scale;
        let h = VHF * scale;
        draw_texture_ex(
            &target.texture,
            (screen_width() - w) * 0.5,
            (screen_height() - h) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                flip_y: true,
                ..Default::default()
            },
        );

        next_frame().await;
    }
}
