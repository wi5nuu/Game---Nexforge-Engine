use crate::game::{GameState, MiniGame, MAX_AMMO};
use crate::enemy::Enemy;
use nexforge_renderer::ui::UiRect;

pub struct HudElement {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub color: [f32; 4],
}

pub struct HudOutput {
    pub elements: Vec<HudElement>,
    pub rects: Vec<UiRect>,
}

const PANEL: [f32; 4] = [0.05, 0.05, 0.10, 0.78];
const PANEL_ACCENT: [f32; 4] = [0.10, 0.10, 0.20, 0.85];
const PANEL_WARN: [f32; 4] = [0.20, 0.04, 0.04, 0.85];
const BAR_BG: [f32; 4] = [0.15, 0.05, 0.05, 0.90];
const BAR_HP_HI: [f32; 4] = [0.15, 0.85, 0.30, 1.00];
const BAR_HP_LO: [f32; 4] = [0.90, 0.18, 0.18, 1.00];
const WHITE: [f32; 4] = [1.00, 1.00, 1.00, 1.00];
const DIM: [f32; 4] = [0.60, 0.60, 0.65, 1.00];
const GOLD: [f32; 4] = [1.00, 0.80, 0.15, 1.00];
const GREEN: [f32; 4] = [0.25, 1.00, 0.45, 1.00];
const RED: [f32; 4] = [1.00, 0.25, 0.25, 1.00];
const CYAN: [f32; 4] = [0.35, 0.85, 1.00, 1.00];

pub fn build_hud(game: &MiniGame, sw: f32, sh: f32, vp: [[f32; 4]; 4], enemies: &[Enemy]) -> HudOutput {
    let mut out = match game.state {
        GameState::Menu => build_menu(sw, sh),
        GameState::Playing => build_playing(game, sw, sh, vp, enemies),
        GameState::Paused => build_paused(game, sw, sh),
        GameState::GameOver | GameState::Victory => build_end(game, sw, sh),
    };
    if game.flash_timer > 0.0 {
        let a = (game.flash_timer / 0.3).min(1.0) as f32;
        out.rects.push(UiRect::new(0.0, 0.0, sw, sh, [0.8, 0.0, 0.0, a * 0.22]));
        out.elements.push(txt("!! HIT !!", sw * 0.5 - 40.0, sh * 0.5 - 55.0, 26.0, [1.0, 0.2, 0.2, a]));
    }
    out
}

fn build_menu(sw: f32, sh: f32) -> HudOutput {
    let mut r = vec![];
    let mut t = vec![];
    let title_x = sw * 0.5 - 240.0;
    let title_y = sh * 0.12;
    r.push(UiRect::new(title_x - 16.0, title_y - 10.0, 492.0, 56.0, PANEL_ACCENT));
    t.push(txt("NEXFORGE DUNGEON", title_x, title_y, 36.0, GOLD));
    t.push(txt("Wave-based FPS survival — clear all waves to escape.", sw * 0.5 - 260.0, sh * 0.22, 14.0, DIM));
    let obj_x = sw * 0.5 - 270.0;
    let obj_y = sh * 0.29;
    r.push(UiRect::new(obj_x - 12.0, obj_y - 8.0, 552.0, 62.0, PANEL));
    t.push(txt("HOW TO PLAY", obj_x, obj_y, 13.0, CYAN));
    t.push(txt("Eliminate all enemies in each wave to advance. Collect gold relics", obj_x, obj_y + 18.0, 13.0, WHITE));
    t.push(txt("for score (+100) and green relics to restore health (+25 HP).", obj_x, obj_y + 34.0, 13.0, WHITE));
    let ctrl_x = sw * 0.5 - 270.0;
    let ctrl_y = sh * 0.44;
    r.push(UiRect::new(ctrl_x - 12.0, ctrl_y - 8.0, 552.0, 120.0, PANEL));
    t.push(txt("CONTROLS", ctrl_x, ctrl_y, 13.0, CYAN));
    let col1 = ctrl_x; let col2 = ctrl_x + 230.0;
    let row = |i: f32| ctrl_y + 18.0 + i * 20.0;
    t.push(txt("W A S D", col1, row(0.0), 13.0, WHITE)); t.push(txt("Move", col2, row(0.0), 13.0, DIM));
    t.push(txt("Mouse", col1, row(1.0), 13.0, WHITE)); t.push(txt("Look / Aim", col2, row(1.0), 13.0, DIM));
    t.push(txt("L-Click", col1, row(2.0), 13.0, WHITE)); t.push(txt("Shoot", col2, row(2.0), 13.0, DIM));
    t.push(txt("R", col1, row(3.0), 13.0, WHITE)); t.push(txt("Reload", col2, row(3.0), 13.0, DIM));
    t.push(txt("Shift", col1, row(4.0), 13.0, WHITE)); t.push(txt("Sprint", col2, row(4.0), 13.0, DIM));
    t.push(txt("ESC", col1, row(5.0), 13.0, WHITE)); t.push(txt("Pause / Quit", col2, row(5.0), 13.0, DIM));
    let btn_x = sw * 0.5 - 175.0;
    let btn_y = sh * 0.73;
    r.push(UiRect::new(btn_x - 12.0, btn_y - 6.0, 364.0, 38.0, [0.15, 0.25, 0.60, 0.90]));
    t.push(txt("PRESS ENTER TO START", btn_x, btn_y, 20.0, WHITE));
    t.push(txt("Press ESC to quit", sw * 0.5 - 90.0, sh * 0.82, 13.0, DIM));
    HudOutput { elements: t, rects: r }
}

fn build_playing(game: &MiniGame, sw: f32, sh: f32, vp: [[f32; 4]; 4], enemies: &[Enemy]) -> HudOutput {
    let mut r = vec![];
    let mut t = vec![];

    // ── Left stats panel ──
    let px = 12.0; let py = 12.0; let pw = 200.0; let ph = 120.0;
    r.push(UiRect::new(px, py, pw, ph, if game.health <= game.max_health * 0.35 { PANEL_WARN } else { PANEL }));
    let hp_frac = (game.health / game.max_health).clamp(0.0, 1.0);
    let hp_color = if game.health > game.max_health * 0.35 { BAR_HP_HI } else { BAR_HP_LO };
    let hp_text_color = if game.health > game.max_health * 0.35 { GREEN } else { RED };
    t.push(txt("HP", px + 8.0, py + 8.0, 11.0, DIM));
    t.push(txt(&format!("{:.0}/{:.0}", game.health, game.max_health), px + 110.0, py + 8.0, 11.0, hp_text_color));
    r.push(UiRect::new(px + 8.0, py + 26.0, pw - 16.0, 10.0, BAR_BG));
    r.push(UiRect::new(px + 8.0, py + 26.0, (pw - 16.0) * hp_frac, 10.0, hp_color));
    t.push(txt("SCORE", px + 8.0, py + 46.0, 11.0, DIM));
    t.push(txt(&format!("{}", game.score), px + 80.0, py + 46.0, 18.0, GOLD));
    t.push(txt("RELOAD", px + 8.0, py + 74.0, 11.0, DIM));
    if game.reloading {
        let progress = 1.0 - (game.reload_timer / 1.5).clamp(0.0, 1.0) as f32;
        r.push(UiRect::new(px + 8.0, py + 90.0, pw - 16.0, 8.0, BAR_BG));
        r.push(UiRect::new(px + 8.0, py + 90.0, (pw - 16.0) * progress, 8.0, CYAN));
    }

    // ── Wave info ──
    let wave_text = format!("WAVE {}/{}", game.current_wave, 3);
    let wave_w = 120.0; let wave_x = sw * 0.5 - wave_w * 0.5;
    r.push(UiRect::new(wave_x - 6.0, 12.0, wave_w + 12.0, 30.0, PANEL));
    t.push(txt(&wave_text, wave_x, 16.0, 18.0, GOLD));
    if game.enemies_alive > 0 {
        t.push(txt(&format!("{} remaining", game.enemies_alive), wave_x + 8.0, 34.0, 11.0, RED));
    } else if game.wave_cooldown > 0.0 {
        t.push(txt("Wave complete! Get ready...", wave_x - 30.0, 34.0, 11.0, GREEN));
    }

    // ── Timer ──
    let minutes = (game.game_time as u32) / 60;
    let seconds = (game.game_time as u32) % 60;
    r.push(UiRect::new(sw - 106.0, 12.0, 96.0, 30.0, PANEL));
    t.push(txt(&format!("{:02}:{:02}", minutes, seconds), sw - 100.0, 16.0, 18.0, WHITE));

    // ── Crosshair ──
    let cx = sw * 0.5; let cy = sh * 0.5;
    let gap = 6.0; let arm = 10.0; let thick = 2.0;
    let cross_col = [1.0, 1.0, 1.0, 0.70];
    r.push(UiRect::new(cx - gap - arm, cy - thick * 0.5, arm, thick, cross_col));
    r.push(UiRect::new(cx + gap, cy - thick * 0.5, arm, thick, cross_col));
    r.push(UiRect::new(cx - thick * 0.5, cy - gap - arm, thick, arm, cross_col));
    r.push(UiRect::new(cx - thick * 0.5, cy + gap, thick, arm, cross_col));

    // ── Attack flash ring ──
    if game.attack_flash > 0.0 {
        let t_frac = (game.attack_flash / 0.18).clamp(0.0, 1.0) as f32;
        let alpha = t_frac * 0.85;
        let ring_r = 22.0 + (1.0 - t_frac) * 20.0;
        let attack_col = [1.0, 0.85, 0.1, alpha];
        r.push(UiRect::new(cx - ring_r, cy - ring_r, ring_r * 2.0, 3.0, attack_col));
        r.push(UiRect::new(cx - ring_r, cy + ring_r - 3.0, ring_r * 2.0, 3.0, attack_col));
        r.push(UiRect::new(cx - ring_r, cy - ring_r, 3.0, ring_r * 2.0, attack_col));
        r.push(UiRect::new(cx + ring_r - 3.0, cy - ring_r, 3.0, ring_r * 2.0, attack_col));
    }

    // ── Ammo bar ──
    let ammo_w = 260.0; let ammo_h = 14.0;
    let ammo_x = sw * 0.5 - ammo_w * 0.5;
    let ammo_y = sh - 30.0;
    let ammo_frac = game.ammo as f32 / MAX_AMMO as f32;
    let ammo_color = if game.ammo > MAX_AMMO / 3 { [0.15, 0.70, 1.0, 1.0] } else { [1.0, 0.2, 0.2, 1.0] };
    r.push(UiRect::new(ammo_x - 1.0, ammo_y - 1.0, ammo_w + 2.0, ammo_h + 2.0, [0.0, 0.0, 0.0, 0.8]));
    r.push(UiRect::new(ammo_x, ammo_y, ammo_w, ammo_h, [0.1, 0.1, 0.15, 0.9]));
    if ammo_frac > 0.0 {
        r.push(UiRect::new(ammo_x, ammo_y, ammo_w * ammo_frac, ammo_h, ammo_color));
    }
    t.push(txt(&format!("{}/{}", game.ammo, MAX_AMMO), ammo_x + ammo_w + 8.0, ammo_y, 14.0, ammo_color));
    if game.reloading {
        let rx = sw * 0.5 - 40.0;
        t.push(txt("RELOADING...", rx, ammo_y - 18.0, 14.0, CYAN));
    }

    // ── Gun view model (bottom-right) ──
    let gun_base_x = sw - 180.0;
    let gun_base_y = sh - 10.0;
    let recoil_offset = (game.attack_flash / 0.18).min(1.0) as f32 * 8.0;
    let gx = gun_base_x;
    let gy = gun_base_y - recoil_offset;
    // Grip
    let grip_w = 20.0; let grip_h = 60.0;
    r.push(UiRect::new(gx - grip_w * 0.5, gy - grip_h, grip_w, grip_h, [0.35, 0.3, 0.25, 0.95]));
    // Barrel
    let barrel_w = 12.0; let barrel_h = 80.0;
    r.push(UiRect::new(gx - barrel_w * 0.5 - 4.0, gy - grip_h - barrel_h, barrel_w, barrel_h, [0.25, 0.25, 0.28, 0.95]));
    // Trigger guard
    r.push(UiRect::new(gx - 14.0, gy - 20.0, 6.0, 14.0, [0.15, 0.15, 0.18, 0.9]));
    // Muzzle flash
    if game.attack_flash > 0.08 {
        let flash_alpha = ((game.attack_flash - 0.08) / 0.1).min(1.0) as f32;
        let flash_w = 18.0; let flash_h = 18.0;
        r.push(UiRect::new(gx - flash_w * 0.5 - 4.0, gy - grip_h - barrel_h - flash_h, flash_w, flash_h, [1.0, 0.9, 0.2, flash_alpha * 0.9]));
        r.push(UiRect::new(gx - 6.0, gy - grip_h - barrel_h - flash_h, 12.0, 28.0, [1.0, 0.7, 0.1, flash_alpha * 0.4]));
    }

    // ── Message ──
    if !game.message.is_empty() && game.message_timer > 0.0 && game.message_timer < 998.0 {
        let alpha = (game.message_timer.min(0.5) / 0.5) as f32;
        let msg_w = 380.0_f32.max(game.message.len() as f32 * 9.5);
        let msg_x = (sw - msg_w) * 0.5;
        let msg_y = sh * 0.28;
        r.push(UiRect::new(msg_x - 14.0, msg_y - 8.0, msg_w + 28.0, 38.0, [0.05, 0.05, 0.15, 0.85 * alpha]));
        t.push(txt(&game.message, msg_x, msg_y + 2.0, 17.0, [1.0, 1.0, 0.4, alpha]));
    }

    // ── Enemy health bars ──
    for enemy in enemies {
        if !enemy.active { continue; }
        if let Some((sx, sy)) = world_to_screen([enemy.position[0], enemy.position[1] + 2.8, enemy.position[2]], vp, sw, sh) {
            let bar_w = 64.0; let bar_h = 8.0;
            let bx = sx - bar_w * 0.5; let by = sy;
            let hp_frac = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
            let fill_col = if enemy.chase_mode { [1.0, 0.12, 0.05, 0.95] } else { [0.80, 0.25, 0.10, 0.90] };
            r.push(UiRect::new(bx - 1.0, by - 1.0, bar_w + 2.0, bar_h + 2.0, [0.0, 0.0, 0.0, 0.85]));
            r.push(UiRect::new(bx, by, bar_w, bar_h, [0.20, 0.04, 0.04, 0.85]));
            if hp_frac > 0.0 { r.push(UiRect::new(bx, by, bar_w * hp_frac, bar_h, fill_col)); }
            if enemy.chase_mode { t.push(txt("!", sx - 4.0, by - 18.0, 16.0, [1.0, 0.2, 0.05, 0.95])); }
        }
    }

    HudOutput { elements: t, rects: r }
}

fn build_paused(game: &MiniGame, sw: f32, sh: f32) -> HudOutput {
    let mut r = vec![];
    let mut t = vec![];
    r.push(UiRect::new(0.0, 0.0, sw, sh, [0.0, 0.0, 0.05, 0.55]));
    let card_w = 420.0; let card_h = 220.0;
    let card_x = (sw - card_w) * 0.5; let card_y = (sh - card_h) * 0.5;
    r.push(UiRect::new(card_x, card_y, card_w, card_h, PANEL_ACCENT));
    r.push(UiRect::new(card_x, card_y, card_w, 4.0, [0.35, 0.55, 1.0, 1.0]));
    t.push(txt("PAUSED", card_x + card_w * 0.5 - 68.0, card_y + 18.0, 32.0, WHITE));
    r.push(UiRect::new(card_x + 20.0, card_y + 60.0, card_w - 40.0, 1.0, [0.3, 0.3, 0.5, 0.6]));
    let stats_x = card_x + 30.0;
    t.push(txt(&format!("Score    {}", game.score), stats_x, card_y + 72.0, 16.0, GOLD));
    t.push(txt(&format!("Wave     {}/{}", game.current_wave, 3), stats_x, card_y + 94.0, 16.0, CYAN));
    t.push(txt(&format!("Kills    {}", game.kills), stats_x, card_y + 116.0, 16.0, RED));
    t.push(txt(&format!("Health   {:.0}/{:.0}", game.health, game.max_health), stats_x, card_y + 138.0, 16.0, GREEN));
    r.push(UiRect::new(card_x + 20.0, card_y + 162.0, card_w - 40.0, 1.0, [0.3, 0.3, 0.5, 0.6]));
    t.push(txt("ESC - Resume", card_x + 30.0, card_y + 175.0, 14.0, WHITE));
    t.push(txt("ENTER - Restart", card_x + card_w * 0.5 + 10.0, card_y + 175.0, 14.0, DIM));
    HudOutput { elements: t, rects: r }
}

fn build_end(game: &MiniGame, sw: f32, sh: f32) -> HudOutput {
    let mut r = vec![];
    let mut t = vec![];
    let victory = game.state == GameState::Victory;
    let bg = if victory { [0.02, 0.10, 0.04, 0.70] } else { [0.10, 0.02, 0.02, 0.70] };
    r.push(UiRect::new(0.0, 0.0, sw, sh, bg));
    let card_w = 480.0; let card_h = 270.0;
    let card_x = (sw - card_w) * 0.5; let card_y = (sh - card_h) * 0.5;
    let accent = if victory { [0.20, 1.00, 0.40, 1.0] } else { [1.00, 0.20, 0.20, 1.0] };
    r.push(UiRect::new(card_x, card_y, card_w, card_h, PANEL_ACCENT));
    r.push(UiRect::new(card_x, card_y, card_w, 5.0, accent));
    let title = if victory { "VICTORY!" } else { "GAME OVER" };
    let title_x = card_x + card_w * 0.5 - if victory { 95.0 } else { 115.0 };
    t.push(txt(title, title_x, card_y + 22.0, 48.0, accent));
    r.push(UiRect::new(card_x + 20.0, card_y + 82.0, card_w - 40.0, 1.0, [0.3, 0.3, 0.5, 0.6]));
    let lx = card_x + 40.0; let rx = card_x + card_w * 0.5 + 20.0;
    t.push(txt("FINAL SCORE", lx, card_y + 94.0, 12.0, DIM)); t.push(txt(&format!("{}", game.score), rx, card_y + 94.0, 22.0, GOLD));
    t.push(txt("WAVES CLEARED", lx, card_y + 122.0, 12.0, DIM)); t.push(txt(&format!("{}", game.current_wave), rx, card_y + 122.0, 18.0, CYAN));
    t.push(txt("TIME", lx, card_y + 146.0, 12.0, DIM)); t.push(txt(&format!("{:.1}s", game.game_time), rx, card_y + 146.0, 18.0, WHITE));
    t.push(txt("ENEMIES KILLED", lx, card_y + 170.0, 12.0, DIM)); t.push(txt(&format!("{}", game.kills), rx, card_y + 170.0, 18.0, RED));
    r.push(UiRect::new(card_x + 20.0, card_y + 198.0, card_w - 40.0, 1.0, [0.3, 0.3, 0.5, 0.6]));
    let cta_x = card_x + card_w * 0.5 - 170.0;
    r.push(UiRect::new(cta_x - 10.0, card_y + 212.0, 356.0, 34.0, [0.15, 0.20, 0.50, 0.85]));
    t.push(txt("ENTER - Play Again     ESC - Quit", cta_x, card_y + 220.0, 15.0, WHITE));
    HudOutput { elements: t, rects: r }
}

fn txt(text: &str, x: f32, y: f32, scale: f32, color: [f32; 4]) -> HudElement {
    HudElement { text: text.to_string(), x, y, scale, color }
}

fn world_to_screen(pos: [f32; 3], vp: [[f32; 4]; 4], sw: f32, sh: f32) -> Option<(f32, f32)> {
    let v = [pos[0], pos[1], pos[2], 1.0_f32];
    let cx = vp[0][0]*v[0] + vp[1][0]*v[1] + vp[2][0]*v[2] + vp[3][0]*v[3];
    let cy = vp[0][1]*v[0] + vp[1][1]*v[1] + vp[2][1]*v[2] + vp[3][1]*v[3];
    let cw = vp[0][3]*v[0] + vp[1][3]*v[1] + vp[2][3]*v[2] + vp[3][3]*v[3];
    if cw <= 0.01 { return None; }
    let ndcx = cx / cw; let ndcy = cy / cw;
    if ndcx.abs() > 1.05 || ndcy.abs() > 1.05 { return None; }
    Some(((ndcx + 1.0) * 0.5 * sw, (1.0 - ndcy) * 0.5 * sh))
}
