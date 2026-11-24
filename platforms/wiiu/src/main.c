/*
 * Wii U frontend: SDL2 (wut) rendering/input, shared Rust game logic.
 */
#include <SDL2/SDL.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Rust C ABI (crates/flappy-wiiu). */
typedef struct {
    float x, y, w, h;
} SpriteRectOut;

extern void fw_init(int seed);
extern void fw_tick(void);
extern void fw_tap(void);
extern void fw_tap_at(float x, float y);
extern void fw_release_at(float x, float y);
extern void fw_activate(void);
extern int fw_sprite_rect(const unsigned char *name, size_t len, SpriteRectOut *out);

extern int fw_menu(void);
extern int fw_bg_night(void);
extern int fw_bird_x(void);
extern int fw_bird_y(void);
extern float fw_bird_rot(void);
extern float fw_bird_bob(void);
extern int fw_bird_variant(void);
extern int fw_bird_frame(void);
extern int fw_bird_bobbing(void);
extern int fw_land_x(void);
extern int fw_score(void);
extern int fw_pipe_speed(void);
extern int fw_pipes_pending(void);
extern int fw_ready_visible(void);
extern float fw_ready_alpha(void);
extern int fw_over_visible(void);
extern float fw_over_alpha(void);
extern int fw_over_offset(void);
extern int fw_panel_visible(void);
extern int fw_panel_x(void);
extern int fw_panel_y(void);
extern int fw_panel_display(void);
extern int fw_panel_best(void);
extern int fw_panel_state(void);
extern int fw_panel_new(void);
extern int fw_panel_medal(void);
extern int fw_panel_medal_x(void);
extern int fw_panel_medal_y(void);
extern int fw_play_visible(void);
extern int fw_score_visible(void);
extern int fw_rate_visible(void);
extern float fw_fade_value(void);
extern int fw_fade_done(void);
extern float fw_flash_value(void);
extern int fw_flash_done(void);
extern int fw_pipe_x(int i);
extern int fw_pipe_y(int i);
extern int fw_sparkle_active(int i);
extern int fw_sparkle_x(int i);
extern int fw_sparkle_y(int i);
extern int fw_sparkle_frame(int i);

/* Generated from assets/atlas.png by build.sh. */
extern const unsigned char _binary_atlas_rgba_bin_start[];
extern const unsigned char _binary_atlas_rgba_bin_end[];

#define VW 288
#define VH 512
#define WIN_W 1280
#define WIN_H 720
#define SCALE ((float)WIN_H / (float)VH)
#define OFFSET_X ((int)(((float)WIN_W - (float)VW * SCALE) * 0.5f))

#define MENU_PLAY_X ((VW - (116 + 116 + 16)) / 2)
#define MENU_BTN_Y 340
#define MENU_SCORE_X (MENU_PLAY_X + 116 + 16)
#define MENU_RATE_X ((VW - 74) / 2)
#define MENU_RATE_Y 270

#define PIPE_W 52
#define PIPE_H 320
#define PIPE_GAP 96
#define BIRD_SPRITE_OFFSET 14

static SDL_Renderer *renderer;
static SDL_Texture *atlas;
static SDL_GameController *pad;

#define RECT(name, out) fw_sprite_rect((const unsigned char *)(name), strlen(name), (out))

static const char *BIRD_NAMES[3][3] = {
    {"bird0_0", "bird0_1", "bird0_2"},
    {"bird1_0", "bird1_1", "bird1_2"},
    {"bird2_0", "bird2_1", "bird2_2"},
};
static const char *BLINK_NAMES[3] = {"blink_00", "blink_01", "blink_02"};
static const char *MEDAL_NAMES[4] = {"medals_0", "medals_1", "medals_2", "medals_3"};

static void sprite(const char *name, float x, float y, float rotation, float alpha) {
    SpriteRectOut r;
    if (!RECT(name, &r)) {
        return;
    }
    SDL_Rect src = {(int)r.x, (int)r.y, (int)r.w, (int)r.h};
    SDL_Rect dst = {
        OFFSET_X + (int)(x * SCALE + 0.5f),
        (int)(y * SCALE + 0.5f),
        (int)(r.w * SCALE + 0.5f),
        (int)(r.h * SCALE + 0.5f),
    };
    float a = alpha < 0.0f ? 0.0f : (alpha > 1.0f ? 1.0f : alpha);
    SDL_SetTextureAlphaMod(atlas, (Uint8)(a * 255.0f));
    if (rotation != 0.0f) {
        SDL_Point c = {dst.w / 2, dst.h / 2};
        SDL_RenderCopyEx(renderer, atlas, &src, &dst, rotation, &c, SDL_FLIP_NONE);
    } else {
        SDL_RenderCopy(renderer, atlas, &src, &dst);
    }
}

static void bird_sprite(int variant, int frame, float x, float y, float rotation) {
    int f = frame;
    if (f < 0 || f > 2) {
        f = 1;
    }
    sprite(BIRD_NAMES[variant < 0 || variant > 2 ? 0 : variant][f], x, y, rotation, 1.0f);
}

static void draw_pipe_pair(int x, int y) {
    sprite("pipe_up", (float)x, (float)y, 0.0f, 1.0f);
    sprite("pipe_down", (float)x, (float)(y - PIPE_H - PIPE_GAP), 0.0f, 1.0f);
}

static void draw_hud(int score) {
    char name[24];
    int total = 2;
    int x, y;
    int i, n;
    int digits[10];
    int v;

    v = score < 0 ? 0 : score;
    n = 0;
    if (v == 0) {
        digits[0] = 0;
        n = 1;
    } else {
        while (v > 0 && n < 10) {
            digits[n++] = v % 10;
            v /= 10;
        }
    }
    for (i = 0; i < n; i++) {
        SpriteRectOut r;
        snprintf(name, sizeof name, "font_%03d", 48 + digits[i]);
        if (RECT(name, &r)) {
            total += (int)r.w - 4;
        }
    }
    x = 144 - total / 2;
    y = 100 - 44 / 2;
    for (i = n - 1; i >= 0; i--) {
        SpriteRectOut r;
        snprintf(name, sizeof name, "font_%03d", 48 + digits[i]);
        if (!RECT(name, &r)) {
            continue;
        }
        sprite(name, (float)x, (float)y, 0.0f, 1.0f);
        x += (int)r.w - 4;
    }
}

static void draw_number(int value, int right_x, int y) {
    int v = value < 0 ? 0 : value;
    int x = right_x - 16;
    int count = 10;
    int leading = 1;
    char name[24];
    while (count > 0) {
        if (v > 0 || leading) {
            snprintf(name, sizeof name, "number_score_%02d", v % 10);
            sprite(name, (float)x, (float)y, 0.0f, 1.0f);
            x -= 16;
            v /= 10;
            leading = 0;
        }
        count--;
    }
}

static void draw_scene(void) {
    int i;
    int menu = fw_menu();
    int pending = fw_pipes_pending();

    sprite(fw_bg_night() ? "bg_night" : "bg_day", 0.0f, 0.0f, 0.0f, 1.0f);

    if (!menu && pending <= 0) {
        for (i = 0; i < 3; i++) {
            if (fw_pipe_x(i) < VW) {
                draw_pipe_pair(fw_pipe_x(i), fw_pipe_y(i));
            }
        }
    }

    if (menu) {
        SpriteRectOut r;
        float title_w = 0.0f;
        if (RECT("title", &r)) {
            title_w = r.w;
        }
        sprite("title", (288.0f - title_w) / 2.0f, 150.0f, 0.0f, 1.0f);
        bird_sprite(fw_bird_variant(), fw_bird_frame(), 120.0f - BIRD_SPRITE_OFFSET,
                    218.0f - BIRD_SPRITE_OFFSET + fw_bird_bob(), 0.0f);
        sprite("land", (float)fw_land_x(), 400.0f, 0.0f, 1.0f);
    } else {
        if (fw_over_visible()) {
            float w = 0.0f;
            SpriteRectOut r;
            if (RECT("text_game_over", &r)) {
                w = r.w;
            }
            sprite("text_game_over", (288.0f - w) / 2.0f, (float)(fw_over_offset() + 130),
                   0.0f, fw_over_alpha());
        } else {
            draw_hud(fw_score());
        }

        bird_sprite(fw_bird_variant(), fw_bird_frame(),
                    (float)(fw_bird_x() - BIRD_SPRITE_OFFSET),
                    (float)(fw_bird_y() - BIRD_SPRITE_OFFSET) + fw_bird_bob(),
                    fw_bird_rot());
        sprite("land", (float)fw_land_x(), 400.0f, 0.0f, 1.0f);
    }

    if (fw_ready_visible()) {
        float w = 0.0f;
        SpriteRectOut r;
        float a = fw_ready_alpha();
        if (RECT("text_ready", &r)) {
            w = r.w;
        }
        sprite("text_ready", (288.0f - w) / 2.0f, 146.0f, 0.0f, a);
        if (RECT("tutorial", &r)) {
            w = r.w;
        }
        sprite("tutorial", (288.0f - w) / 2.0f, 220.0f, 0.0f, a);
    }

    if (menu) {
        float w = 0.0f;
        SpriteRectOut r;
        if (RECT("brand_copyright", &r)) {
            w = r.w;
        }
        sprite("brand_copyright", (288.0f - w) / 2.0f, 418.0f, 0.0f, 1.0f);
    }

    if (fw_play_visible()) {
        sprite("button_play", (float)MENU_PLAY_X, (float)MENU_BTN_Y, 0.0f, 1.0f);
    }
    if (fw_score_visible()) {
        sprite("button_score", (float)MENU_SCORE_X, (float)MENU_BTN_Y, 0.0f, 1.0f);
    }
    if (fw_rate_visible()) {
        sprite("button_rate", (float)MENU_RATE_X, (float)MENU_RATE_Y, 0.0f, 1.0f);
    }

    if (fw_panel_visible()) {
        int px = fw_panel_x();
        int py = fw_panel_y();
        sprite("score_panel", (float)px, (float)py, 0.0f, 1.0f);
        draw_number(fw_panel_display(), px + 210, py + 36);
        draw_number(fw_panel_best(), px + 210, py + 78);
        if (fw_panel_new()) {
            sprite("new", (float)(px + 142), (float)(py + 60), 0.0f, 1.0f);
        }
        if (fw_panel_medal() >= 0) {
            sprite(MEDAL_NAMES[fw_panel_medal()], (float)fw_panel_medal_x(),
                   (float)fw_panel_medal_y(), 0.0f, 1.0f);
        }
    }

    for (i = 0; i < 10; i++) {
        if (fw_sparkle_active(i)) {
            char name[16];
            int f = fw_sparkle_frame(i);
            if (f < 0 || f > 2) {
                f = 0;
            }
            snprintf(name, sizeof name, "%s", BLINK_NAMES[f]);
            sprite(name, (float)(fw_sparkle_x(i) - 4), (float)(fw_sparkle_y(i) - 4), 0.0f, 1.0f);
        }
    }

    if (!fw_fade_done() || fw_fade_value() != 0.0f) {
        SDL_SetRenderDrawColor(renderer, 0, 0, 0, (Uint8)(fw_fade_value() * 255.0f));
        SDL_Rect full = {OFFSET_X, 0, (int)((float)VW * SCALE), WIN_H};
        SDL_RenderFillRect(renderer, &full);
    }
    if (!fw_flash_done() || fw_flash_value() != 0.0f) {
        SDL_SetRenderDrawColor(renderer, 255, 255, 255, (Uint8)(fw_flash_value() * 255.0f));
        SDL_Rect full = {OFFSET_X, 0, (int)((float)VW * SCALE), WIN_H};
        SDL_RenderFillRect(renderer, &full);
    }
}

static void tap(void) {
    if (fw_menu() || (fw_panel_visible() && fw_panel_state() == 2)) {
        fw_activate();
    } else {
        fw_tap();
    }
}

int main(int argc, char **argv) {
    Uint32 tick;
    int running = 1;
    int prev_a = 0, prev_start = 0, prev_up = 0, prev_r = 0;
    SDL_Event e;
    SDL_Rect viewport = {OFFSET_X, 0, (int)((float)VW * SCALE), WIN_H};

    (void)argc;
    (void)argv;

    SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY, "0");
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMECONTROLLER | SDL_INIT_EVENTS) != 0) {
        return 1;
    }
    SDL_Window *window = SDL_CreateWindow("Flappy Rust", SDL_WINDOWPOS_CENTERED,
                                          SDL_WINDOWPOS_CENTERED, WIN_W, WIN_H, SDL_WINDOW_SHOWN);
    if (!window) {
        SDL_Quit();
        return 2;
    }
    renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    if (!renderer) {
        renderer = SDL_CreateRenderer(window, -1, 0);
    }
    if (!renderer) {
        SDL_DestroyWindow(window);
        SDL_Quit();
        return 3;
    }
    SDL_RenderSetClipRect(renderer, &viewport);
    SDL_SetRenderDrawBlendMode(renderer, SDL_BLENDMODE_BLEND);

    atlas = SDL_CreateTexture(renderer, SDL_PIXELFORMAT_RGBA32, SDL_TEXTUREACCESS_STATIC, 1024, 1024);
    if (!atlas) {
        SDL_DestroyRenderer(renderer);
        SDL_DestroyWindow(window);
        SDL_Quit();
        return 4;
    }
    SDL_SetTextureBlendMode(atlas, SDL_BLENDMODE_BLEND);
    SDL_UpdateTexture(atlas, NULL, _binary_atlas_rgba_bin_start,
                      (int)(_binary_atlas_rgba_bin_end - _binary_atlas_rgba_bin_start) / 1024);

    for (int i = 0; i < SDL_NumJoysticks(); i++) {
        if (SDL_IsGameController(i)) {
            pad = SDL_GameControllerOpen(i);
            if (pad) {
                break;
            }
        }
    }

    fw_init((int)SDL_GetTicks());

    while (running) {
        while (SDL_PollEvent(&e)) {
            switch (e.type) {
            case SDL_QUIT:
                running = 0;
                break;
            case SDL_CONTROLLERDEVICEADDED:
                if (!pad && SDL_IsGameController(e.cdevice.which)) {
                    pad = SDL_GameControllerOpen(e.cdevice.which);
                }
                break;
            case SDL_CONTROLLERDEVICEREMOVED:
                if (pad) {
                    SDL_GameControllerClose(pad);
                    pad = NULL;
                }
                break;
            case SDL_FINGERDOWN: {
                float vx = ((float)e.tfinger.x * WIN_W - OFFSET_X) / SCALE;
                float vy = (float)e.tfinger.y * WIN_H / SCALE;
                if (fw_menu() || (fw_panel_visible() && fw_panel_state() == 2)) {
                    fw_activate();
                } else {
                    fw_tap_at(vx, vy);
                }
                break;
            }
            case SDL_FINGERUP: {
                float vx = ((float)e.tfinger.x * WIN_W - OFFSET_X) / SCALE;
                float vy = (float)e.tfinger.y * WIN_H / SCALE;
                fw_release_at(vx, vy);
                break;
            }
            default:
                break;
            }
        }

        if (pad) {
            int a = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_A);
            int st = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_START);
            int up = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_DPAD_UP);
            int r = SDL_GameControllerGetButton(pad, SDL_CONTROLLER_BUTTON_RIGHTSHOULDER);
            if ((a && !prev_a) || (st && !prev_start) || (up && !prev_up) || (r && !prev_r)) {
                tap();
            }
            prev_a = a;
            prev_start = st;
            prev_up = up;
            prev_r = r;
        }

        fw_tick();

        SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
        SDL_RenderClear(renderer);
        draw_scene();
        SDL_RenderPresent(renderer);
        {
            Uint32 elapsed = SDL_GetTicks() - tick;
            if (elapsed < 16) {
                SDL_Delay(16 - elapsed);
            }
        }
        tick = SDL_GetTicks();
    }

    if (pad) {
        SDL_GameControllerClose(pad);
    }
    SDL_DestroyTexture(atlas);
    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    SDL_Quit();
    return 0;
}
