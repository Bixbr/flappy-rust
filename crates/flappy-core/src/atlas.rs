pub const ATLAS_SIZE: f32 = 1024.0;
pub const MAX_SPRITES: usize = 96;

#[derive(Clone, Copy, Debug, Default)]
pub struct SpriteRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy)]
pub struct Sprite {
    pub name: &'static str,
    pub rect: SpriteRect,
}

const EMPTY: Sprite = Sprite {
    name: "",
    rect: SpriteRect {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    },
};

pub struct AtlasMap {
    pub sprites: [Sprite; MAX_SPRITES],
    pub len: usize,
}

impl AtlasMap {
    pub fn parse(txt: &'static str) -> Self {
        let mut map = Self {
            sprites: [EMPTY; MAX_SPRITES],
            len: 0,
        };
        for line in txt.lines() {
            let mut fields = line.split_whitespace();
            let Some(name) = fields.next() else { continue };
            let (Some(w), Some(h), Some(u), Some(v)) = (
                fields.next().and_then(|s| s.parse::<f32>().ok()),
                fields.next().and_then(|s| s.parse::<f32>().ok()),
                fields.next().and_then(|s| s.parse::<f32>().ok()),
                fields.next().and_then(|s| s.parse::<f32>().ok()),
            ) else {
                continue;
            };
            if map.len >= MAX_SPRITES {
                break;
            }
            map.sprites[map.len] = Sprite {
                name,
                rect: SpriteRect {
                    x: ((u * ATLAS_SIZE + 0.5) as i32 as f32),
                    y: ((v * ATLAS_SIZE + 0.5) as i32 as f32),
                    w,
                    h,
                },
            };
            map.len += 1;
        }
        map
    }

    pub fn get(&self, name: &str) -> Option<SpriteRect> {
        self.sprites[..self.len]
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.rect)
    }

    pub fn rect(&self, name: &str) -> SpriteRect {
        self.get(name).unwrap_or(SpriteRect {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_sprites() {
        let map = AtlasMap::parse(include_str!("../../../assets/atlas.txt"));
        assert_eq!(map.len, 71);
        let bird = map.rect("bird0_1");
        assert_eq!(bird.w, 48.0);
        assert_eq!(bird.h, 48.0);
        assert_eq!(bird.x, 56.0);
        assert_eq!(bird.y, 970.0);
    }
}
