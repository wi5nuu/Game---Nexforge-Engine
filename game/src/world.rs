use crate::bullet::Bullets;
use crate::enemy::Enemies;
use crate::game::{ARENA_SIZE, WALL_HEIGHT, WALL_THICKNESS};
use crate::items::Items;
use nexforge_renderer::scene::{Scene, SceneObject};

pub fn build_arena(device: &wgpu::Device, scene: &mut Scene) {
    let half = ARENA_SIZE / 2.0;
    let hw = half + WALL_THICKNESS / 2.0;
    let wall_w = ARENA_SIZE + WALL_THICKNESS * 2.0;
    let wall_color = [0.35, 0.35, 0.45];
    let wall_h = WALL_HEIGHT;

    let walls = [
        ([hw, wall_h / 2.0, 0.0], [WALL_THICKNESS, wall_h, wall_w], wall_color),
        ([-hw, wall_h / 2.0, 0.0], [WALL_THICKNESS, wall_h, wall_w], wall_color),
        ([0.0, wall_h / 2.0, hw], [wall_w, wall_h, WALL_THICKNESS], wall_color),
        ([0.0, wall_h / 2.0, -hw], [wall_w, wall_h, WALL_THICKNESS], wall_color),
    ];
    for (pos, scale, color) in &walls {
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout, *pos, *color, *scale));
    }

    let floor_tiles = 7;
    let tile_size = ARENA_SIZE / floor_tiles as f32;
    for ix in 0..floor_tiles {
        for iz in 0..floor_tiles {
            let x = -half + tile_size * ix as f32 + tile_size / 2.0;
            let z = -half + tile_size * iz as f32 + tile_size / 2.0;
            let alt = (ix + iz) % 2 == 0;
            let color = if alt { [0.18, 0.20, 0.18] } else { [0.15, 0.18, 0.15] };
            scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout, [x, -0.05, z], color, [tile_size * 0.9, 0.05, tile_size * 0.9]));
        }
    }

    let pillar_positions = [
        (-half + 1.5, -half + 1.5), (-half + 1.5, half - 1.5),
        (half - 1.5, -half + 1.5), (half - 1.5, half - 1.5),
    ];
    for (px, pz) in &pillar_positions {
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout, [*px, WALL_HEIGHT / 2.0, *pz], [0.5, 0.4, 0.25], [0.4, WALL_HEIGHT, 0.4]));
    }

    let cover_positions = [
        (-2.0, -2.0), (2.0, 2.0), (-3.0, 3.0), (3.0, -3.0), (0.0, 0.0),
    ];
    for (cx, cz) in &cover_positions {
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout, [*cx, 0.8, *cz], [0.35, 0.3, 0.25], [1.2, 1.6, 1.2]));
    }
}

pub fn build_item_objects(device: &wgpu::Device, scene: &mut Scene, items: &Items) {
    for item in &items.items {
        if !item.collected {
            let float_offset = item.phase.sin() * 0.25;
            let color = if item.is_health { [0.1, 1.0, 0.2] } else { [1.0, 0.82, 0.0] };
            scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
                [item.position[0], item.position[1] + 0.6 + float_offset, item.position[2]],
                color, [0.5, 0.5, 0.5]));
        }
    }
}

pub fn build_enemy_objects(device: &wgpu::Device, scene: &mut Scene, enemies: &Enemies) {
    for enemy in &enemies.enemies {
        if !enemy.active { continue; }
        let enemy_type = enemy.enemy_type;
        let health_ratio = enemy.health / enemy.max_health;
        let stunned = enemy.stun_timer > 0.0;
        let (body_color, head_color, eye_color, arm_color, leg_color) = if stunned {
            ([1.0, 0.95, 0.1], [1.0, 1.0, 0.5], [1.0, 0.3, 0.0], [1.0, 0.9, 0.1], [1.0, 0.85, 0.1])
        } else {
            match enemy_type {
                0 => ([0.15, 0.55, 0.9], [0.3, 0.7, 1.0], [1.0, 1.0, 0.6], [0.1, 0.45, 0.8], [0.1, 0.4, 0.75]),
                1 => ([0.8, 0.2, 0.1], [0.9, 0.35, 0.2], [1.0, 0.3, 0.1], [0.7, 0.15, 0.08], [0.65, 0.12, 0.08]),
                _ => ([0.35, 0.28, 0.22], [0.5, 0.42, 0.35], [1.0, 0.1, 0.05], [0.45, 0.38, 0.32], [0.4, 0.35, 0.3]),
            }
        };
        let chase_brighten: f32 = if enemy.chase_mode && !stunned { 0.25 } else { 0.0 };
        let body = [
            (body_color[0] + chase_brighten).min(1.0),
            (body_color[1] + chase_brighten).min(1.0),
            (body_color[2] + chase_brighten).min(1.0),
        ];
        let head = [
            (head_color[0] + chase_brighten).min(1.0),
            (head_color[1] + chase_brighten).min(1.0),
            (head_color[2] + chase_brighten).min(1.0),
        ];
        let dim = if health_ratio < 0.33 { 0.85 } else if health_ratio < 0.66 { 0.95 } else { 1.0 };
        let bob = enemy.phase.sin() * 0.08;
        let base_y = enemy.position[1] + 0.9 + bob;
        let px = enemy.position[0];
        let pz = enemy.position[2];
        let body_w = if enemy_type == 2 { 0.8 * dim } else { 0.65 * dim };
        let body_h = 1.8 * dim;
        let body_d = if enemy_type == 2 { 0.8 * dim } else { 0.65 * dim };
        let arm_w = 0.25 * dim;
        let arm_h = 1.3 * dim;
        let arm_d = 0.25 * dim;
        let leg_w = 0.3 * dim;
        let leg_h = 0.5 * dim;
        let leg_d = 0.3 * dim;
        let eye_s = 0.12 * dim;
        let arm_gap = body_w / 2.0 + arm_w / 2.0;
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px, base_y, pz], body, [body_w, body_h, body_d]));
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px - arm_gap, base_y - 0.15, pz], arm_color, [arm_w, arm_h, arm_d]));
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px + arm_gap, base_y - 0.15, pz], arm_color, [arm_w, arm_h, arm_d]));
        let leg_gap = leg_w / 2.0 + 0.05;
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px - leg_gap, base_y - body_h / 2.0 - leg_h / 2.0, pz], leg_color, [leg_w, leg_h, leg_d]));
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px + leg_gap, base_y - body_h / 2.0 - leg_h / 2.0, pz], leg_color, [leg_w, leg_h, leg_d]));
        let head_y = base_y + body_h / 2.0 + 0.4 * dim;
        let head_s = 0.55 * dim;
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px, head_y, pz], head, [head_s, head_s, head_s]));
        let eye_z = if enemy_type == 2 { 0.25 * dim } else { 0.2 * dim };
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px - 0.12 * dim, head_y + 0.05 * dim, pz + eye_z], eye_color, [eye_s, eye_s, 0.05]));
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [px + 0.12 * dim, head_y + 0.05 * dim, pz + eye_z], eye_color, [eye_s, eye_s, 0.05]));
        if enemy_type == 2 && health_ratio > 0.5 {
            scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
                [px - body_w / 2.0 - 0.1 * dim, base_y + 0.6 * dim, pz], [0.5, 0.4, 0.3], [0.4 * dim, 0.15 * dim, 0.5 * dim]));
            scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
                [px + body_w / 2.0 + 0.1 * dim, base_y + 0.6 * dim, pz], [0.5, 0.4, 0.3], [0.4 * dim, 0.15 * dim, 0.5 * dim]));
        }
    }
}

pub fn build_bullet_objects(device: &wgpu::Device, scene: &mut Scene, bullets: &Bullets) {
    for bullet in &bullets.bullets {
        let color = if bullet.from_player { [1.0, 0.85, 0.1] } else { [1.0, 0.15, 0.05] };
        scene.add_object(SceneObject::new_scaled(device, &scene.bind_group_layout,
            [bullet.position[0], bullet.position[1], bullet.position[2]], color, [0.08, 0.08, 0.15]));
    }
}
