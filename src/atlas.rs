use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct SpriteRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct Atlas {
    pub tex: Texture2D,
    sprites: HashMap<String, SpriteRect>,
}

impl Atlas {
    pub async fn load() -> Self {
        let tex = load_texture("assets/atlas.png")
            .await
            .expect("failed to load assets/atlas.png");
        tex.set_filter(FilterMode::Nearest);

        let txt =
            std::fs::read_to_string("assets/atlas.txt").expect("failed to load assets/atlas.txt");
        let mut sprites = HashMap::new();
        for line in txt.lines() {
            let p: Vec<&str> = line.split_whitespace().collect();
            if p.len() < 7 {
                continue;
            }
            let w: f32 = p[1].parse().unwrap();
            let h: f32 = p[2].parse().unwrap();
            let u: f32 = p[3].parse().unwrap();
            let v: f32 = p[4].parse().unwrap();
            sprites.insert(
                p[0].to_string(),
                SpriteRect {
                    x: (u * 1024.0).round(),
                    y: (v * 1024.0).round(),
                    w,
                    h,
                },
            );
        }

        Self { tex, sprites }
    }

    pub fn rect(&self, name: &str) -> SpriteRect {
        *self
            .sprites
            .get(name)
            .unwrap_or_else(|| panic!("sprite not found in atlas: {name}"))
    }

    pub fn width(&self, name: &str) -> f32 {
        self.rect(name).w
    }

    pub fn draw(&self, name: &str, x: f32, y: f32, scale: f32, rotation_deg: f32, color: Color) {
        let r = self.rect(name);
        let w = r.w * scale;
        let h = r.h * scale;
        draw_texture_ex(
            &self.tex,
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                source: Some(Rect::new(r.x, r.y, r.w, r.h)),
                rotation: rotation_deg.to_radians(),
                pivot: Some(vec2(x + w * 0.5, y + h * 0.5)),
                flip_x: false,
                flip_y: false,
            },
        );
    }
}
