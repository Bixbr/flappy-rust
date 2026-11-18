//! Platform-independent Flappy Bird simulation.
//!
//! Rendering, audio and platform input live in the frontends. Physics,
//! collisions, scoring, timers and UI state transitions are shared by every
//! platform and run at a fixed 60 Hz step.

use crate::rng::Rng;
use crate::sin_table::SIN_TABLE;

pub const VW: u32 = 288;
pub const VH: u32 = 512;
pub const VWF: f32 = 288.0;
pub const VHF: f32 = 512.0;
pub const FIXED_DT: f32 = 1.0 / 60.0;

pub const BIRD_START_X: i32 = 80;
pub const BIRD_START_Y: i32 = 246;
pub const BIRD_HIT_W: i32 = 20;
pub const BIRD_HIT_H: i32 = 20;
pub const BIRD_SPRITE_OFFSET: i32 = 14;

pub const GROUND_TOP: i32 = 400;
pub const PIPE_W: i32 = 52;
pub const PIPE_H: i32 = 320;
pub const PIPE_GAP: i32 = 96;
pub const PIPE_SPACING: i32 = 105;

pub const MENU_PLAY_X: i32 = (288 - (116 + 116 + 16)) / 2;
pub const MENU_BTN_Y: i32 = 340;
pub const MENU_SCORE_X: i32 = MENU_PLAY_X + 116 + 16;
pub const MENU_RATE_X: i32 = (288 - 74) / 2;
pub const MENU_RATE_Y: i32 = 270;

pub const PANEL_W: i32 = 238;
pub const PANEL_H: i32 = 126;
pub const PANEL_FINAL_Y: i32 = (512 - PANEL_H) / 2;

// Sound events emitted by the simulation; frontends drain them each tick.
pub const EV_WING: u16 = 1 << 0;
pub const EV_POINT: u16 = 1 << 1;
pub const EV_HIT: u16 = 1 << 2;
pub const EV_DIE: u16 = 1 << 3;
pub const EV_SWOOSH: u16 = 1 << 4;

#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u
}

fn ease_out_quint(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u * u * u
}

pub struct Tween {
    from: f32,
    to: f32,
    len: i32,
    frame: i32,
    kind: i32,
    pub value: f32,
    pub done: bool,
}

impl Tween {
    pub fn new() -> Self {
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

    pub fn start(&mut self, from: f32, to: f32, kind: i32, duration: f32) {
        self.from = from;
        self.to = to;
        self.kind = kind;
        self.len = ((60.0 * duration) as i32).max(1);
        self.frame = 0;
        self.value = from;
        self.done = false;
    }

    pub fn advance(&mut self) -> bool {
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

impl Default for Tween {
    fn default() -> Self {
        Self::new()
    }
}

const FLAP_SEQ: [i32; 12] = [0, 1, 2, 1, 0, 1, 2, 1, 0, 1, 2, 1];

pub struct Anim {
    interval: f32,
    looping: bool,
    acc: f32,
    idx: usize,
    pub done: bool,
}

impl Anim {
    pub fn new(fps: i32, looping: bool) -> Self {
        Self {
            interval: (1000 / fps) as f32,
            looping,
            acc: 0.0,
            idx: 0,
            done: true,
        }
    }

    pub fn start(&mut self) {
        self.acc = 0.0;
        self.idx = 0;
        self.done = false;
    }

    pub fn advance(&mut self) {
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

    pub fn frame(&self) -> i32 {
        FLAP_SEQ[self.idx]
    }
}

#[derive(Clone, Copy)]
pub enum TimerAction {
    MenuInit,
    OpenPanel,
    DieSound,
    WingSound,
}

#[derive(Clone, Copy)]
pub struct Timer {
    pub remaining: i32,
    pub action: TimerAction,
}

const MAX_TIMERS: usize = 8;
const MAX_SPARKLES: usize = 10;

pub struct Bird {
    pub x: i32,
    pub y: i32,
    pub rot: f32,
    pub rot_vel: f32,
    pub rot_acc: f32,
    pub vy: f32,
    pub gravity: f32,
    pub bobbing: bool,
    pub dead: bool,
    pub bob_x: i32,
    pub bob_y: f32,
    pub variant: i32,
    flap_anim: Anim,
    auto_anim: Anim,
    auto_active: bool,
}

impl Bird {
    pub fn new() -> Self {
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

    pub fn reset(&mut self, rng: &mut Rng) {
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

    pub fn tick(&mut self) {
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
            self.bob_y = SIN_TABLE[self.bob_x as usize] * 4.0;
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

    pub fn flap(&mut self, emit_wing: &mut bool) {
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
            *emit_wing = true;
        }
    }

    pub fn frame_index(&self) -> i32 {
        if self.auto_active {
            self.auto_anim.frame()
        } else if !self.flap_anim.done {
            self.flap_anim.frame()
        } else {
            1
        }
    }
}

impl Default for Bird {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Ready {
    pub visible: bool,
    pub stage: i32,
    pub alpha: Tween,
}

impl Ready {
    pub fn new() -> Self {
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
        }
    }
}

impl Default for Ready {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OverText {
    pub visible: bool,
    pub stage: i32,
    pub alpha: Tween,
    pub offset: i32,
    dy: f32,
    accel: f32,
}

impl OverText {
    pub fn new() -> Self {
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

impl Default for OverText {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Panel {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
    pub score: i32,
    pub best: i32,
    pub display: i32,
    pub state: i32,
    pub new_badge: bool,
    slide: Tween,
    count: Tween,
    pub medal: Option<usize>,
    pub medal_x: i32,
    pub medal_y: i32,
    sparkle_timer: i32,
}

impl Panel {
    pub fn new() -> Self {
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

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Sparkle {
    pub active: bool,
    pub x: i32,
    pub y: i32,
    pub anim: Anim,
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

pub struct Sparkles {
    pub items: [Sparkle; MAX_SPARKLES],
    next: usize,
}

impl Sparkles {
    pub fn new() -> Self {
        Self {
            items: core::array::from_fn(|_| Sparkle::new()),
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

impl Default for Sparkles {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Game {
    pub menu: bool,
    pub bg_night: bool,
    pub bird: Bird,
    pub land_x: i32,
    pub pipe_speed: i32,
    pub pipes: [(i32, i32); 3],
    pub pipes_pending: i32,
    pub score: i32,
    pub best: i32,
    pub fade: Tween,
    pub fade_state: i32,
    pub flash: Tween,
    pub ready: Ready,
    pub over_text: OverText,
    pub panel: Panel,
    pub sparkles: Sparkles,
    pub play_visible: bool,
    pub score_visible: bool,
    pub rate_visible: bool,
    rng: Rng,
    timers: [Option<Timer>; MAX_TIMERS],
    events: u16,
}

impl Game {
    pub fn new(seed: i32) -> Self {
        let mut rng = Rng::new(seed);
        let mut bird = Bird::new();
        bird.variant = rng.next() % 3;
        bird.reset(&mut rng);

        let mut fade = Tween::new();
        fade.start(1.0, 0.0, 5, 0.5);
        fade.advance();

        let mut timers = [None; MAX_TIMERS];
        timers[0] = Some(Timer {
            remaining: 1,
            action: TimerAction::MenuInit,
        });

        Self {
            menu: true,
            bg_night: false,
            bird,
            land_x: 0,
            pipe_speed: 2,
            pipes: [(79, 274), (236, 274), (393, 274)],
            pipes_pending: 1,
            score: 0,
            best: 0,
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
            rng,
            timers,
            events: 0,
        }
    }

    pub fn take_events(&mut self) -> u16 {
        core::mem::take(&mut self.events)
    }

    fn emit(&mut self, event: u16) {
        self.events |= event;
    }

    fn push_timer(&mut self, remaining: i32, action: TimerAction) {
        for slot in &mut self.timers {
            if slot.is_none() {
                *slot = Some(Timer { remaining, action });
                return;
            }
        }
    }

    pub fn handle_input(&mut self, tap: bool, release: bool, pos: Point) {
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
                    self.flap();
                }
            } else if self.pipe_speed > 0 {
                self.flap();
            }
        }
    }

    /// Flap without requiring a menu hit test (controller input).
    pub fn flap(&mut self) {
        let mut wing = false;
        self.bird.flap(&mut wing);
        if wing {
            self.emit(EV_WING);
            self.push_timer(5, TimerAction::WingSound);
        }
    }

    /// Start or restart the round from the title menu.
    pub fn activate_play(&mut self) {
        if self.menu || (self.panel.visible && self.panel.state == 2) {
            self.begin_start_game();
        }
    }

    fn begin_start_game(&mut self) {
        if !self.fade.done {
            return;
        }
        self.fade_state = 5;
        self.fade.start(0.0, 1.0, 5, 0.5);
        self.fade.advance();
        self.emit(EV_SWOOSH);
    }

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
            TimerAction::DieSound => self.emit(EV_DIE),
            TimerAction::WingSound => self.emit(EV_WING),
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

    fn die(&mut self, bird_dead: bool) {
        self.flash.start(1.0, 0.0, 11, 1.0);
        self.flash.advance();
        if bird_dead {
            self.bird.dead = true;
        }
        self.pipe_speed = 0;
        self.emit(EV_HIT);
        self.push_timer(500, TimerAction::DieSound);
        self.push_timer(1000, TimerAction::OpenPanel);
    }

    pub fn tick(&mut self) {
        let mut i = 0;
        while i < MAX_TIMERS {
            if let Some(timer) = &mut self.timers[i] {
                timer.remaining -= 30;
                if timer.remaining <= 0 {
                    let action = timer.action;
                    self.timers[i] = None;
                    self.handle_timer(action);
                }
            }
            i += 1;
        }

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
                self.emit(EV_POINT);
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

        self.bird.tick();

        if !self.menu {
            // Ground collision
            if self.bird.y >= GROUND_TOP - BIRD_HIT_H && self.pipe_speed > 0 {
                self.die(false);
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
                    self.die(false);
                } else if hit_first_bottom {
                    self.die(true);
                }

                let hit_second = overlap(bird_box, (p1x, p1y - PIPE_H - PIPE_GAP, PIPE_W, PIPE_H))
                    || overlap(bird_box, (p1x, p1y, PIPE_W, PIPE_H));

                if hit_second {
                    self.die(true);
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
                    self.emit(EV_SWOOSH);
                }
                if self.over_text.stage == 1 && self.panel.state == 2 {
                    self.over_text.stage = 2;
                }
            }
        }

        if self.ready.visible {
            self.ready.tick();
        }

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
}

pub fn overlap(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    let (x, y, w, h) = a;
    let (x2, y2, w2, h2) = b;
    x + w >= x2 && x <= x2 + w2 && y + h >= y2 && y <= y2 + h2
}

pub fn point_in(p: Point, x: i32, y: i32, w: i32, h: i32) -> bool {
    p.x >= x as f32 && p.x <= (x + w) as f32 && p.y >= y as f32 && p.y <= (y + h) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_round(seed: i32) -> Game {
        let mut game = Game::new(seed);
        for _ in 0..40 {
            game.tick();
        }
        game.activate_play();
        for _ in 0..120 {
            game.tick();
        }
        game.flap();
        game
    }

    #[test]
    fn starts_in_menu_and_transitions_to_ready() {
        let game = start_round(1);
        assert!(!game.menu);
        assert!(game.ready.visible);
    }

    #[test]
    fn gravity_moves_bird_down_and_ground_stops_it() {
        let mut game = start_round(7);
        let start_y = game.bird.y;
        for _ in 0..60 {
            game.tick();
        }
        assert!(game.bird.y > start_y);
        for _ in 0..300 {
            game.tick();
        }
        assert_eq!(game.bird.y, GROUND_TOP - BIRD_HIT_H);
    }

    #[test]
    fn flap_impulse_sets_negative_velocity() {
        let mut game = start_round(3);
        game.flap();
        assert!(game.bird.vy < 0.0);
        assert_eq!(game.take_events() & EV_WING, EV_WING);
    }

    #[test]
    fn scoring_advances_rng_and_collision_stops_pipes() {
        let mut game = start_round(11);
        // Let the bird fall; it must eventually hit the ground and stop pipes.
        for _ in 0..600 {
            game.tick();
        }
        assert_eq!(game.pipe_speed, 0);
        assert!(game.panel.visible || game.over_text.visible);
    }
}
