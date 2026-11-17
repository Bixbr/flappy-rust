mod atlas;
mod rng;

use atlas::Atlas;
use macroquad::audio::{load_sound, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use rng::Rng;

// ---------------------------------------------------------------------------
// Game constants (exactly as in the decompiled com.dotgears classes)
// ---------------------------------------------------------------------------
const VW: u32 = 288;
const VH: u32 = 512;
const VWF: f32 = 288.0;
const VHF: f32 = 512.0;
const FIXED_DT: f32 = 1.0 / 60.0;

const BIRD_START_X: i32 = 80;
const BIRD_START_Y: i32 = 246;
const BIRD_HIT_W: i32 = 20;
const BIRD_HIT_H: i32 = 20;
const BIRD_SPRITE_OFFSET: i32 = 14;

const GROUND_TOP: i32 = 400;
const PIPE_W: i32 = 52;
const PIPE_H: i32 = 320;
const PIPE_GAP: i32 = 96;
const PIPE_SPACING: i32 = 105;

const MENU_PLAY_X: i32 = (288 - (116 + 116 + 16)) / 2;
const MENU_BTN_Y: i32 = 340;
const MENU_SCORE_X: i32 = MENU_PLAY_X + 116 + 16;
const MENU_RATE_X: i32 = (288 - 74) / 2;
const MENU_RATE_Y: i32 = 270;

const PANEL_W: i32 = 238;
const PANEL_H: i32 = 126;
const PANEL_FINAL_Y: i32 = (512 - PANEL_H) / 2;

// ---------------------------------------------------------------------------
// Tween / animation helpers
// ---------------------------------------------------------------------------
fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u
}

fn ease_out_quint(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u * u * u
}

struct Tween {
    from: f32,
    to: f32,
    len: i32,
    frame: i32,
    kind: i32,
    pub value: f32,
    pub done: bool,
}

impl Tween {
    fn new() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            len: 1,
            frame: 0,
            kind: 0,
            value: 0.0,
            done: true,
        }
    }

    fn start(&mut self, from: f32, to: f32, kind: i32, duration: f32) {
        self.from = from;
        self.to = to;
        self.kind = kind;
        self.len = ((60.0 * duration) as i32).max(1);
        self.frame = 0;
        self.value = from;
        self.done = false;
    }

    fn advance(&mut self) -> bool {
        if self.done {
            return false;
        }
        self.frame += 1;
        let t = self.frame as f32 / self.len as f32;
        let eased = match self.kind {
            5 => ease_out_cubic(t),
            11 => ease_out_quint(t),
            _ => t,
        };
        self.value = self.from + (self.to - self.from) * eased;
        if self.frame >= self.len {
            self.value = self.to;
            self.done = true;
            return true;
        }
        false
    }
}

const FLAP_SEQ: [i32; 12] = [0, 1, 2, 1, 0, 1, 2, 1, 0, 1, 2, 1];

struct Anim {
    interval: f32,
    looping: bool,
    acc: f32,
    idx: usize,
    done: bool,
}

impl Anim {
    fn new(fps: i32, looping: bool) -> Self {
        Self {
            interval: (1000 / fps) as f32,
            looping,
            acc: 0.0,
            idx: 0,
            done: true,
        }
    }

    fn start(&mut self) {
        self.acc = 0.0;
        self.idx = 0;
        self.done = false;
    }

    fn advance(&mut self) {
        if self.done {
            return;
        }
        self.acc += 15.0;
        if self.acc >= self.interval {
            self.acc = 0.0;
            self.idx += 1;
            if self.idx >= FLAP_SEQ.len() {
                if self.looping {
                    self.idx = 0;
                } else {
                    self.idx = 0;
                    self.done = true;
                }
            }
        }
    }

    fn frame(&self) -> i32 {
        FLAP_SEQ[self.idx]
    }
}

// ---------------------------------------------------------------------------
// Timers / sounds
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
enum TimerAction {
    MenuInit,
    OpenPanel,
    DieSound,
    WingSound,
}

struct Timer {
    remaining: i32,
    action: TimerAction,
}

#[derive(Clone, Copy)]
enum Sfx {
    Wing,
    Point,
    Hit,
    Die,
    Swoosh,
}

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
            wing: load_sound("assets/sfx_wing.ogg").await.ok(),
            point: load_sound("assets/sfx_point.ogg").await.ok(),
            hit: load_sound("assets/sfx_hit.ogg").await.ok(),
            die: load_sound("assets/sfx_die.ogg").await.ok(),
            swoosh: load_sound("assets/sfx_swooshing.ogg").await.ok(),
        }
    }

    fn play(&self, sfx: Sfx) {
        let sound = match sfx {
            Sfx::Wing => &self.wing,
            Sfx::Point => &self.point,
            Sfx::Hit => &self.hit,
            Sfx::Die => &self.die,
            Sfx::Swoosh => &self.swoosh,
        };
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
}

// ---------------------------------------------------------------------------
// Bird (com.dotgears.flappy.a)
// ---------------------------------------------------------------------------
struct Bird {
    x: i32,
    y: i32,
    rot: f32,
    rot_vel: f32,
    rot_acc: f32,
    vy: f32,
    gravity: f32,
    bobbing: bool,
    dead: bool,
    bob_x: i32,
    bob_y: f32,
    variant: i32,
    flap_anim: Anim,
    auto_anim: Anim,
    auto_active: bool,
}

impl Bird {
    fn new() -> Self {
        Self {
            x: BIRD_START_X,
            y: BIRD_START_Y,
            rot: 0.0,
            rot_vel: 0.0,
            rot_acc: 0.4,
            vy: 0.0,
            gravity: 1.0,
            bobbing: true,
            dead: false,
            bob_x: 0,
            bob_y: 0.0,
            variant: 0,
            flap_anim: Anim::new(30, false),
            auto_anim: Anim::new(10, true),
            auto_active: true,
        }
    }

    fn reset(&mut self, rng: &mut Rng) {
        self.x = BIRD_START_X;
        self.y = BIRD_START_Y;
        self.rot = 0.0;
        self.vy = 0.0;
        self.gravity = 1.0;
        self.rot_vel = 0.0;
        self.rot_acc = 0.4;
        self.dead = false;
        self.bobbing = true;
        self.bob_x = 0;
        self.bob_y = 0.0;
        self.auto_anim.start();
        self.auto_active = true;
        self.variant = rng.next() % 3;
    }

    fn tick(&mut self, sin_table: &[f32; 360]) {
        if self.auto_active {
            self.auto_anim.advance();
        } else {
            self.flap_anim.advance();
        }

        if self.bobbing {
            self.bob_x += 8;
            if self.bob_x == 360 {
                self.bob_x = 0;
            }
            self.bob_y = sin_table[self.bob_x as usize] * 4.0;
            return;
        }

        self.bob_y = 0.0;
        self.vy += self.gravity;
        if self.vy > 8.0 {
            self.vy = 8.0;
        }
        self.y += self.vy as i32;
        if self.y > GROUND_TOP - BIRD_HIT_H {
            self.y = GROUND_TOP - BIRD_HIT_H;
            self.gravity = 0.0;
            self.vy = 0.0;
        }

        self.rot += self.rot_vel;
        self.rot_vel += self.rot_acc;
        self.rot = self.rot.clamp(-20.0, 90.0);
    }

    fn flap(&mut self, timers: &mut Vec<Timer>) {
        if self.bobbing {
            self.bobbing = false;
        }
        if self.y >= 0 && !self.dead {
            self.flap_anim.start();
            self.auto_active = false;
            self.vy = -5.0;
            self.gravity = 0.3;
            self.rot_vel = -10.0;
            self.rot_acc = 0.4;
            timers.push(Timer {
                remaining: 5,
                action: TimerAction::WingSound,
            });
        }
    }

    fn frame_index(&self) -> i32 {
        if self.auto_active {
            self.auto_anim.frame()
        } else if !self.flap_anim.done {
            self.flap_anim.frame()
        } else {
            1
        }
    }
}

// ---------------------------------------------------------------------------
// Ready overlay (com.dotgears.flappy.f)
// ---------------------------------------------------------------------------
struct Ready {
    visible: bool,
    stage: i32,
    alpha: Tween,
}

impl Ready {
    fn new() -> Self {
        Self {
            visible: false,
            stage: 0,
            alpha: Tween::new(),
        }
    }

    fn show(&mut self) {
        self.visible = true;
        self.stage = 0;
        self.alpha.start(0.0, 1.0, 0, 0.5);
    }

    fn dismiss(&mut self) {
        self.stage = 2;
        self.alpha.start(1.0, 0.0, 0, 0.5);
    }

    fn hide(&mut self) {
        self.visible = false;
        self.stage = 0;
        self.alpha = Tween::new();
    }

    fn tick(&mut self) {
        if !self.visible {
            return;
        }
        self.alpha.advance();
        if self.stage == 0 && self.alpha.done {
            self.stage = 1;
        } else if self.stage == 2 && self.alpha.done {
            self.stage = 2;
        }
    }
}

// ---------------------------------------------------------------------------
// "Game over" text (com.dotgears.flappy.e)
// ---------------------------------------------------------------------------
struct OverText {
    visible: bool,
    stage: i32,
    alpha: Tween,
    offset: i32,
    dy: f32,
    accel: f32,
}

impl OverText {
    fn new() -> Self {
        Self {
            visible: false,
            stage: 0,
            alpha: Tween::new(),
            offset: -1,
            dy: -2.0,
            accel: 0.25,
        }
    }

    fn show(&mut self) {
        self.visible = true;
        self.stage = 0;
        self.alpha.start(0.0, 1.0, 11, 1.0);
        self.offset = -1;
        self.dy = -2.0;
        self.accel = 0.25;
    }

    fn hide(&mut self) {
        self.visible = false;
        self.stage = 0;
        self.alpha = Tween::new();
    }

    fn tick(&mut self) -> bool {
        self.alpha.advance();
        if self.offset < 0 {
            self.offset = (self.offset as f32 + self.dy) as i32;
            self.dy += self.accel;
        } else {
            self.offset = 0;
        }
        if self.stage == 0 && self.alpha.done {
            self.stage = 1;
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Score panel (com.dotgears.p)
// ---------------------------------------------------------------------------
struct Panel {
    visible: bool,
    x: i32,
    y: i32,
    score: i32,
    best: i32,
    display: i32,
    state: i32,
    new_badge: bool,
    slide: Tween,
    count: Tween,
    medal: Option<usize>,
    medal_x: i32,
    medal_y: i32,
    sparkle_timer: i32,
}

impl Panel {
    fn new() -> Self {
        Self {
            visible: false,
            x: (288 - PANEL_W) / 2,
            y: 504,
            score: 0,
            best: 0,
            display: 0,
            state: 0,
            new_badge: false,
            slide: Tween::new(),
            count: Tween::new(),
            medal: None,
            medal_x: 0,
            medal_y: 0,
            sparkle_timer: 30,
        }
    }

    fn open(&mut self, score: i32, best: i32) {
        self.score = score;
        self.best = best;
        self.display = 0;
        self.visible = true;
        self.new_badge = false;
        self.x = (288 - PANEL_W) / 2;
        self.y = 504;
        self.slide.start(504.0, PANEL_FINAL_Y as f32, 11, 0.5);
        self.state = 0;
        self.medal = None;
        self.sparkle_timer = 30;
    }

    fn hide(&mut self) {
        self.visible = false;
        self.state = 0;
        self.medal = None;
    }

    fn tick(&mut self, rng: &mut Rng) -> Option<(i32, i32)> {
        if !self.visible {
            return None;
        }

        if !self.slide.done {
            self.slide.advance();
        }

        match self.state {
            0 => {
                self.y = self.slide.value as i32;
                if self.slide.done {
                    if self.score <= 0 {
                        self.state = 2;
                    } else {
                        self.state = 1;
                        self.count.start(0.0, self.score as f32, 0, 0.5);
                    }
                }
            }
            1 => {
                if !self.count.done {
                    self.count.advance();
                }
                self.display = self.count.value as i32;
                if self.count.done {
                    self.state = 2;
                    if self.display > self.best {
                        self.best = self.display;
                        self.new_badge = true;
                    }
                    self.medal = if self.display >= 40 {
                        Some(0)
                    } else if self.display >= 30 {
                        Some(1)
                    } else if self.display >= 20 {
                        Some(2)
                    } else if self.display >= 10 {
                        Some(3)
                    } else {
                        None
                    };
                    self.medal_x = self.x + 32;
                    self.medal_y = self.y + 44;
                    self.sparkle_timer = 30;
                }
            }
            _ => {
                if self.medal.is_some() {
                    self.sparkle_timer -= 1;
                    if self.sparkle_timer <= 0 {
                        self.sparkle_timer = 30;
                        return Some((
                            self.medal_x - 3 + rng.range(0, 44 + 6),
                            self.medal_y - 3 + rng.range(0, 44 + 6),
                        ));
                    }
                }
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Medal sparkles (com.dotgears.e / particle pool)
// ---------------------------------------------------------------------------
struct Sparkle {
    active: bool,
    x: i32,
    y: i32,
    anim: Anim,
}

impl Sparkle {
    fn new() -> Self {
        Self {
            active: false,
            x: 0,
            y: 0,
            anim: Anim::new(10, false),
        }
    }
}

struct Sparkles {
    items: Vec<Sparkle>,
    next: usize,
}

impl Sparkles {
    fn new() -> Self {
        Self {
            items: (0..10).map(|_| Sparkle::new()).collect(),
            next: 0,
        }
    }

    fn reset(&mut self) {
        for s in &mut self.items {
            s.active = false;
            s.anim.done = true;
        }
    }

    fn spawn(&mut self, x: i32, y: i32) {
        let index = self.next;
        self.next = (self.next + 1) % self.items.len();
        let s = &mut self.items[index];
        s.active = true;
        s.x = x;
        s.y = y;
        s.anim.start();
    }

    fn tick(&mut self) {
        for s in &mut self.items {
            if s.active {
                s.anim.advance();
                if s.anim.done {
                    s.active = false;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Game scene (com.dotgears.flappy.c)
// ---------------------------------------------------------------------------
struct Game {
    atlas: Atlas,
    sounds: Sounds,
    rng: Rng,
    sin_table: [f32; 360],

    menu: bool,
    bg_night: bool,
    bird: Bird,
    land_x: i32,
    pipe_speed: i32,
    pipes: [(i32, i32); 3],
    pipes_pending: i32,
    score: i32,
    best: i32,

    timers: Vec<Timer>,
    fade: Tween,
    fade_state: i32,
    flash: Tween,

    ready: Ready,
    over_text: OverText,
    panel: Panel,
    sparkles: Sparkles,

    play_visible: bool,
    score_visible: bool,
    rate_visible: bool,
}

impl Game {
    fn new(atlas: Atlas, sounds: Sounds, seed: i32) -> Self {
        let mut rng = Rng::new(seed);
        let mut sin_table = [0.0f32; 360];
        for (i, value) in sin_table.iter_mut().enumerate() {
            *value = ((i as f32) * 3.1415925 / 180.0).sin();
        }

        let mut bird = Bird::new();
        bird.variant = rng.next() % 3;
        bird.reset(&mut rng);

        let mut fade = Tween::new();
        fade.start(1.0, 0.0, 5, 0.5);
        fade.advance();

        Self {
            atlas,
            sounds,
            rng,
            sin_table,
            menu: true,
            bg_night: false,
            bird,
            land_x: 0,
            pipe_speed: 2,
            pipes: [(79, 274), (236, 274), (393, 274)],
            pipes_pending: 1,
            score: 0,
            best: 0,
            timers: vec![Timer {
                remaining: 1,
                action: TimerAction::MenuInit,
            }],
            fade,
            fade_state: 0,
            flash: Tween::new(),
            ready: Ready::new(),
            over_text: OverText::new(),
            panel: Panel::new(),
            sparkles: Sparkles::new(),
            play_visible: false,
            score_visible: false,
            rate_visible: false,
        }
    }

    // -- input ------------------------------------------------------------
    fn handle_input(&mut self, tap: bool, release: bool, pos: Vec2) {
        if self.menu {
            if release && self.play_visible && point_in(pos, MENU_PLAY_X, MENU_BTN_Y, 116, 70) {
                self.begin_start_game();
            }
            return;
        }

        if release
            && self.play_visible
            && self.panel.state == 2
            && point_in(pos, MENU_PLAY_X, MENU_BTN_Y, 116, 70)
        {
            self.begin_start_game();
            return;
        }

        if tap {
            if self.bird.bobbing {
                if self.ready.visible && self.ready.stage == 1 {
                    self.ready.dismiss();
                    self.bird.flap(&mut self.timers);
                }
            } else if self.pipe_speed > 0 {
                self.bird.flap(&mut self.timers);
            }
        }
    }

    fn begin_start_game(&mut self) {
        if !self.fade.done {
            return;
        }
        self.fade_state = 5;
        self.fade.start(0.0, 1.0, 5, 0.5);
        self.fade.advance();
        self.sounds.play(Sfx::Swoosh);
    }

    // -- timers -----------------------------------------------------------
    fn handle_timer(&mut self, action: TimerAction) {
        match action {
            TimerAction::MenuInit => {
                self.scene_reset();
                if self.fade.done {
                    self.fade_state = 0;
                    self.fade.start(1.0, 0.0, 5, 0.5);
                    self.fade.advance();
                }
            }
            TimerAction::OpenPanel => {
                self.over_text.show();
            }
            TimerAction::DieSound => self.sounds.play(Sfx::Die),
            TimerAction::WingSound => self.sounds.play(Sfx::Wing),
        }
    }

    fn handle_fade_state(&mut self, state: i32) {
        if state == 5 {
            self.start_game();
        }
    }

    fn scene_reset(&mut self) {
        self.bg_night = self.rng.next() % 10 <= 3;
        self.sparkles.reset();

        self.play_visible = true;
        self.score_visible = true;
        self.rate_visible = true;
        self.ready.hide();
        self.over_text.hide();
        self.panel.hide();

        self.menu = true;
        self.land_x = 0;
        self.bird.reset(&mut self.rng);
        self.pipe_speed = 2;
        self.pipes_pending = 1;
        self.score = 0;
    }

    fn start_game(&mut self) {
        self.scene_reset();

        self.play_visible = false;
        self.score_visible = false;
        self.rate_visible = false;

        self.menu = false;
        if self.fade.done {
            self.fade_state = 0;
            self.fade.start(1.0, 0.0, 5, 0.5);
            self.fade.advance();
        }
        self.land_x = 0;
        self.bird.reset(&mut self.rng);
        self.pipe_speed = 2;
        self.pipes_pending = 1;
        self.score = 0;
        self.ready.show();
    }

    // -- update -----------------------------------------------------------
    fn tick(&mut self) {
        let mut due = Vec::new();
        for timer in &mut self.timers {
            timer.remaining -= 30;
        }
        let mut i = 0;
        while i < self.timers.len() {
            if self.timers[i].remaining <= 0 {
                due.push(self.timers.remove(i).action);
            } else {
                i += 1;
            }
        }
        for action in due {
            self.handle_timer(action);
        }

        // ---- scene update (flappy.c.b) ----
        self.land_x -= self.pipe_speed;
        if self.land_x <= -24 {
            self.land_x = 0;
        }

        if !self.bird.bobbing {
            for pipe in &mut self.pipes {
                pipe.0 -= self.pipe_speed;
            }

            if self.pipe_speed > 0
                && self.pipes_pending <= 0
                && (self.pipes[0].0 == self.bird.x || self.pipes[0].0 == self.bird.x - 1)
            {
                self.score += 1;
                self.sounds.play(Sfx::Point);
            }

            if self.pipes[0].0 < -PIPE_W {
                self.pipes[0] = self.pipes[1];
                self.pipes[1] = self.pipes[2];
                self.pipes[2].0 = self.pipes[1].0 + PIPE_SPACING + PIPE_W;
                self.pipes[2].1 = self.rng.range(180, 360);

                if self.pipes_pending > 0 {
                    self.pipes_pending -= 1;
                    if self.pipes_pending == 0 {
                        self.pipes[1].0 = -PIPE_W;
                        self.pipes[0].0 = -PIPE_W;
                    }
                }
            }
        }

        self.bird.tick(&self.sin_table);

        if !self.menu {
            // Ground collision
            if self.bird.y >= GROUND_TOP - BIRD_HIT_H && self.pipe_speed > 0 {
                self.flash.start(1.0, 0.0, 11, 1.0);
                self.flash.advance();
                self.pipe_speed = 0;
                self.sounds.play(Sfx::Hit);
                self.timers.push(Timer {
                    remaining: 1000,
                    action: TimerAction::OpenPanel,
                });
            }

            // Pipe collisions
            if !self.bird.dead && self.pipes_pending <= 0 && self.pipe_speed > 0 {
                let (p0x, p0y) = self.pipes[0];
                let (p1x, p1y) = self.pipes[1];
                let bird_box = (self.bird.x, self.bird.y, BIRD_HIT_W, BIRD_HIT_H);

                let hit_first_top =
                    overlap(bird_box, (p0x, p0y - PIPE_H - PIPE_GAP, PIPE_W, PIPE_H));
                let hit_first_bottom = overlap(bird_box, (p0x, p0y, PIPE_W, PIPE_H));

                if hit_first_top {
                    self.flash.start(1.0, 0.0, 11, 1.0);
                    self.flash.advance();
                    self.pipe_speed = 0;
                    self.sounds.play(Sfx::Hit);
                    self.timers.push(Timer {
                        remaining: 500,
                        action: TimerAction::DieSound,
                    });
                    self.timers.push(Timer {
                        remaining: 1000,
                        action: TimerAction::OpenPanel,
                    });
                } else if hit_first_bottom {
                    self.flash.start(1.0, 0.0, 11, 1.0);
                    self.flash.advance();
                    self.bird.dead = true;
                    self.pipe_speed = 0;
                    self.sounds.play(Sfx::Hit);
                    self.timers.push(Timer {
                        remaining: 500,
                        action: TimerAction::DieSound,
                    });
                    self.timers.push(Timer {
                        remaining: 1000,
                        action: TimerAction::OpenPanel,
                    });
                }

                let hit_second = overlap(bird_box, (p1x, p1y - PIPE_H - PIPE_GAP, PIPE_W, PIPE_H))
                    || overlap(bird_box, (p1x, p1y, PIPE_W, PIPE_H));

                if hit_second {
                    self.flash.start(1.0, 0.0, 11, 1.0);
                    self.flash.advance();
                    self.bird.dead = true;
                    self.pipe_speed = 0;
                    self.sounds.play(Sfx::Hit);
                    self.timers.push(Timer {
                        remaining: 500,
                        action: TimerAction::DieSound,
                    });
                    self.timers.push(Timer {
                        remaining: 1000,
                        action: TimerAction::OpenPanel,
                    });
                }
            }

            if self.panel.visible && self.panel.state == 2 && !self.play_visible {
                self.play_visible = true;
                self.score_visible = true;
            }

            if self.over_text.visible {
                let was_stage = self.over_text.stage;
                let advanced = self.over_text.tick();
                if was_stage == 0 && advanced {
                    self.panel.open(self.score, self.best);
                    self.sounds.play(Sfx::Swoosh);
                }
                if self.over_text.stage == 1 && self.panel.state == 2 {
                    self.over_text.stage = 2;
                }
            }
        }

        if self.ready.visible {
            self.ready.tick();
        }

        // ---- engine-level updates after the scene (g.b) ----
        if let Some((x, y)) = self.panel.tick(&mut self.rng) {
            self.sparkles.spawn(x, y);
        }
        if self.panel.best > self.best {
            self.best = self.panel.best;
        }
        self.sparkles.tick();

        if self.fade.advance() {
            let state = self.fade_state;
            if state != 0 {
                self.handle_fade_state(state);
            }
        }
        self.flash.advance();
    }

    // -- rendering --------------------------------------------------------
    fn draw(&self) {
        let bg = if self.bg_night { "bg_night" } else { "bg_day" };
        self.atlas.draw(bg, 0.0, 0.0, 1.0, 0.0, WHITE);

        if !self.menu && self.pipes_pending <= 0 {
            for i in 0..2 {
                self.draw_pipe_pair(self.pipes[i].0, self.pipes[i].1);
            }
            if self.pipes[2].0 < 288 {
                self.draw_pipe_pair(self.pipes[2].0, self.pipes[2].1);
            }
        }

        if self.menu {
            let title_x = (288.0 - self.atlas.width("title")) / 2.0;
            self.atlas.draw("title", title_x, 150.0, 1.0, 0.0, WHITE);

            let x = 120 - BIRD_SPRITE_OFFSET;
            let y = 218 - BIRD_SPRITE_OFFSET;
            let name = format!("bird{}_{}", self.bird.variant, self.bird.frame_index());
            self.atlas
                .draw(&name, x as f32, y as f32 + self.bird.bob_y, 1.0, 0.0, WHITE);

            self.draw_land();
        } else {
            if self.over_text.visible {
                let x = (288.0 - self.atlas.width("text_game_over")) / 2.0;
                self.atlas.draw(
                    "text_game_over",
                    x,
                    (self.over_text.offset + 130) as f32,
                    1.0,
                    0.0,
                    Color::new(1.0, 1.0, 1.0, self.over_text.alpha.value),
                );
            } else {
                self.draw_hud();
            }

            let name = format!("bird{}_{}", self.bird.variant, self.bird.frame_index());
            self.atlas.draw(
                &name,
                (self.bird.x - BIRD_SPRITE_OFFSET) as f32,
                (self.bird.y - BIRD_SPRITE_OFFSET) as f32 + self.bird.bob_y,
                1.0,
                self.bird.rot,
                WHITE,
            );

            self.draw_land();
        }

        if self.ready.visible {
            let alpha = self.ready.alpha.value;
            let color = Color::new(1.0, 1.0, 1.0, alpha);
            let x = (288.0 - self.atlas.width("text_ready")) / 2.0;
            self.atlas.draw("text_ready", x, 146.0, 1.0, 0.0, color);
            let x = (288.0 - self.atlas.width("tutorial")) / 2.0;
            self.atlas.draw("tutorial", x, 220.0, 1.0, 0.0, color);
        }

        if self.menu {
            let x = (288.0 - self.atlas.width("brand_copyright")) / 2.0;
            self.atlas
                .draw("brand_copyright", x, 432.0 - 14.0, 1.0, 0.0, WHITE);
        }

        if self.play_visible {
            self.atlas.draw(
                "button_play",
                MENU_PLAY_X as f32,
                MENU_BTN_Y as f32,
                1.0,
                0.0,
                WHITE,
            );
        }
        if self.score_visible {
            self.atlas.draw(
                "button_score",
                MENU_SCORE_X as f32,
                MENU_BTN_Y as f32,
                1.0,
                0.0,
                WHITE,
            );
        }
        if self.rate_visible {
            self.atlas.draw(
                "button_rate",
                MENU_RATE_X as f32,
                MENU_RATE_Y as f32,
                1.0,
                0.0,
                WHITE,
            );
        }

        self.draw_panel();
        self.draw_sparkles();

        if !self.fade.done || self.fade.value != 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                VWF,
                VHF,
                Color::new(0.0, 0.0, 0.0, self.fade.value.clamp(0.0, 1.0)),
            );
        }
        if !self.flash.done || self.flash.value != 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                VWF,
                VHF,
                Color::new(1.0, 1.0, 1.0, self.flash.value.clamp(0.0, 1.0)),
            );
        }
    }

    fn draw_pipe_pair(&self, x: i32, y: i32) {
        self.atlas
            .draw("pipe_up", x as f32, y as f32, 1.0, 0.0, WHITE);
        self.atlas.draw(
            "pipe_down",
            x as f32,
            (y - PIPE_H - PIPE_GAP) as f32,
            1.0,
            0.0,
            WHITE,
        );
    }

    fn draw_land(&self) {
        self.atlas.draw(
            "land",
            self.land_x as f32,
            (512 - 112) as f32,
            1.0,
            0.0,
            WHITE,
        );
    }

    fn draw_hud(&self) {
        let text = self.score.to_string();
        let mut total = 2i32;
        for ch in text.chars() {
            let digit = ch.to_digit(10).unwrap() as i32;
            let name = format!("font_{:03}", 48 + digit);
            total += self.atlas.width(&name) as i32 - 4;
        }
        let mut x = 144 - total / 2;
        let y = 100 - 44 / 2;
        for ch in text.chars() {
            let digit = ch.to_digit(10).unwrap() as i32;
            let name = format!("font_{:03}", 48 + digit);
            let w = self.atlas.width(&name) as i32;
            self.atlas.draw(&name, x as f32, y as f32, 1.0, 0.0, WHITE);
            x += w - 4;
        }
    }

    fn draw_number_score(&self, value: i32, right_x: i32, y: i32) {
        let mut v = value.max(0);
        let mut x = right_x - 16;
        let mut count = 10;
        let mut leading = true;
        while count > 0 {
            if v > 0 || leading {
                let digit = v % 10;
                let name = format!("number_score_0{}", digit);
                self.atlas.draw(&name, x as f32, y as f32, 1.0, 0.0, WHITE);
                x -= 16;
                v /= 10;
                leading = false;
            }
            count -= 1;
        }
    }

    fn draw_panel(&self) {
        if !self.panel.visible {
            return;
        }
        let p = &self.panel;
        self.atlas
            .draw("score_panel", p.x as f32, p.y as f32, 1.0, 0.0, WHITE);
        self.draw_number_score(p.display, p.x + 210, p.y + 36);
        self.draw_number_score(p.best, p.x + 210, p.y + 78);
        if p.new_badge {
            self.atlas.draw(
                "new",
                (p.x + 142) as f32,
                (p.y + 60) as f32,
                1.0,
                0.0,
                WHITE,
            );
        }
        if let Some(medal) = p.medal {
            let name = format!("medals_{}", medal);
            self.atlas
                .draw(&name, p.medal_x as f32, p.medal_y as f32, 1.0, 0.0, WHITE);
        }
    }

    fn draw_sparkles(&self) {
        for s in &self.sparkles.items {
            if s.active && !s.anim.done {
                let name = format!("blink_0{}", s.anim.frame());
                self.atlas
                    .draw(&name, (s.x - 4) as f32, (s.y - 4) as f32, 1.0, 0.0, WHITE);
            }
        }
    }
}

fn overlap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    let (x, y, w, h) = a;
    let (x2, y2, w2, h2) = b;
    x + w >= x2 && x <= x2 + w2 && y + h >= y2 && y <= y2 + h2
}

fn point_in(p: Vec2, x: i32, y: i32, w: i32, h: i32) -> bool {
    p.x >= x as f32 && p.x <= (x + w) as f32 && p.y >= y as f32 && p.y <= (y + h) as f32
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

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64 as i32)
        .unwrap_or(0);

    let mut game = Game::new(atlas, sounds, seed);

    let target = render_target(VW, VH);
    target.texture.set_filter(FilterMode::Nearest);
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VWF, VHF));
    camera.render_target = Some(target.clone());

    let mut accum = 0.0f32;

    loop {
        let input = read_input();
        game.handle_input(input.tap, input.release, input.pos);

        accum += get_frame_time().min(0.1);
        let mut steps = 0;
        while accum >= FIXED_DT && steps < 5 {
            game.tick();
            accum -= FIXED_DT;
            steps += 1;
        }
        if steps == 5 {
            accum = 0.0;
        }

        set_camera(&camera);
        clear_background(BLACK);
        game.draw();

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
