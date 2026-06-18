use std::rc::Rc;

mod game;
mod player;
mod items;
mod enemy;
mod bullet;
mod hud;
mod world;

use bullet::Bullets;
use enemy::Enemies;
use game::{GameState, MiniGame, enemies_for_wave, ENEMY_BULLET_DAMAGE, PLAYER_BULLET_DAMAGE};
use hud::build_hud;
use items::Items;
use player::Player;
use world::{build_arena, build_item_objects, build_enemy_objects, build_bullet_objects};
use nexforge_renderer::pipeline::RenderContext;
use winit::event::{ElementState, Event, WindowEvent, DeviceEvent, MouseButton};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{WindowBuilder, CursorGrabMode};

fn start_game(
    game: &mut MiniGame,
    player: &mut Player,
    items: &mut Items,
    enemies: &mut Enemies,
    bullets: &mut Bullets,
    renderer: &mut RenderContext,
    win: &winit::window::Window,
    cursor_grabbed: &mut bool,
) {
    game.reset();
    player.reset();
    items.reset();
    bullets.clear();
    enemies.clear();
    let count = enemies_for_wave(game.current_wave);
    enemies.spawn_for_wave(count);
    game.enemies_in_wave = count;
    game.enemies_spawned = count;
    game.enemies_alive = count;
    renderer.camera.position = [player.position[0], player.position[1] + 1.1, player.position[2]];
    rebuild_scene(
        renderer.device.as_ref().unwrap(),
        &mut renderer.scene,
        items,
        enemies,
        bullets,
    );
    let _ = win.set_cursor_grab(CursorGrabMode::Confined);
    win.set_cursor_visible(false);
    *cursor_grabbed = true;
}

fn main() {
    env_logger::init();
    log::info!("Nexforge Dungeon starting");

    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => { log::error!("Failed to create event loop: {}", e); return; }
    };

    let window = Rc::new(match WindowBuilder::new()
        .with_title("Nexforge Dungeon")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => { log::error!("Failed to create window: {}", e); return; }
    });
    let mut renderer = RenderContext::new(1280.0 / 720.0);
    if let Err(e) = renderer.initialize(&window) { log::error!("Renderer init failed: {}", e); return; }
    renderer.size = (1280, 720);
    renderer.create_text_renderer();
    renderer.create_ui_renderer();
    let win = window.clone();

    let mut game = MiniGame::new();
    let mut player = Player::new();
    let mut items = Items::new();
    let mut enemies = Enemies::new_for_wave(0);
    let mut bullets = Bullets::new();

    let mut last_time = std::time::Instant::now();
    let mut move_forward = false;
    let mut move_backward = false;
    let mut move_left = false;
    let mut move_right = false;
    let mut sprint = false;
    let mut cursor_grabbed = false;

    if let Some(ref mut scene) = renderer.scene {
        build_arena(renderer.device.as_ref().unwrap(), scene);
    }

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event: window_event, .. } => match window_event {
                    WindowEvent::CloseRequested => { target.exit(); }
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        let pressed = key_event.state == ElementState::Pressed;
                        if let PhysicalKey::Code(code) = key_event.physical_key {
                            match code {
                                KeyCode::Escape => match game.state {
                                    GameState::Playing if cursor_grabbed => {
                                        game.state = GameState::Paused;
                                        let _ = win.set_cursor_grab(CursorGrabMode::None);
                                        win.set_cursor_visible(true);
                                        cursor_grabbed = false;
                                    }
                                    GameState::Paused if !cursor_grabbed => {
                                        game.state = GameState::Playing;
                                        let _ = win.set_cursor_grab(CursorGrabMode::Confined);
                                        win.set_cursor_visible(false);
                                        cursor_grabbed = true;
                                    }
                                    _ => { target.exit(); }
                                },
                                KeyCode::Enter => match game.state {
                                    GameState::Menu | GameState::Paused | GameState::GameOver | GameState::Victory => {
                                        start_game(&mut game, &mut player, &mut items, &mut enemies, &mut bullets, &mut renderer, &win, &mut cursor_grabbed);
                                    }
                                    _ => {}
                                },
                                KeyCode::KeyR => {
                                    if pressed && game.state == GameState::Playing {
                                        game.start_reload();
                                    }
                                }
                                KeyCode::KeyW => move_forward = pressed,
                                KeyCode::KeyS => move_backward = pressed,
                                KeyCode::KeyA => move_left = pressed,
                                KeyCode::KeyD => move_right = pressed,
                                KeyCode::ShiftLeft | KeyCode::ShiftRight => sprint = pressed,
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left && state == ElementState::Pressed && game.state == GameState::Playing {
                            if game.try_shoot() {
                                let fwd = renderer.camera.forward();
                                let origin = [player.position[0], player.position[1] + 0.9, player.position[2]];
                                bullets.fire_player(origin, fwd);
                            }
                        }
                    }
                    WindowEvent::Resized(size) => { renderer.resize((size.width, size.height)); }
                    _ => {}
                },
                Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta: (dx, dy), .. }, .. } => {
                    if cursor_grabbed && game.state == GameState::Playing {
                        renderer.camera.update_mouse(dx as f32, dy as f32);
                    }
                }
                Event::AboutToWait => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f64().min(0.05);
                    last_time = now;

                    match game.state {
                        GameState::Playing => {
                            let horizontal = if move_left { -1.0 } else if move_right { 1.0 } else { 0.0 };
                            let vertical = if move_backward { -1.0 } else if move_forward { 1.0 } else { 0.0 };
                            player.update(dt, horizontal, vertical, sprint, &renderer.camera);
                            renderer.camera.position = [player.position[0], player.position[1] + 1.1, player.position[2]];

                            // Update items
                            items.update(dt);
                            let collected_indices = items.check_collection(player.position);
                            for idx in &collected_indices {
                                if items.items[*idx].is_health { game.heal(25.0); }
                            }

                            // Update bullets
                            bullets.update(dt);

                            // Bullet-player collision
                            let mut player_damage_this_frame = 0.0;
                            bullets.bullets.retain(|b| {
                                if b.from_player { return true; }
                                let dx = b.position[0] - player.position[0];
                                let dz = b.position[2] - player.position[2];
                                let dist = (dx * dx + dz * dz).sqrt();
                                if dist < 1.2 {
                                    if !player.invulnerable { player_damage_this_frame += ENEMY_BULLET_DAMAGE; }
                                    return false;
                                }
                                true
                            });
                            if player_damage_this_frame > 0.0 && player.take_damage(player_damage_this_frame) {
                                game.take_damage(player_damage_this_frame);
                            }

                            // Bullet-enemy collision
                            let mut killed_this_frame = 0;
                            bullets.bullets.retain(|b| {
                                if !b.from_player { return true; }
                                for enemy in &mut enemies.enemies {
                                    if !enemy.active { continue; }
                                    let dx = b.position[0] - enemy.position[0];
                                    let dz = b.position[2] - enemy.position[2];
                                    let dist = (dx * dx + dz * dz).sqrt();
                                    if dist < 1.2 {
                                        enemy.health -= PLAYER_BULLET_DAMAGE;
                                        enemy.stun_timer = 0.3;
                                        if enemy.health <= 0.0 {
                                            enemy.active = false;
                                            game.defeat_enemy();
                                            killed_this_frame += 1;
                                        }
                                        return false;
                                    }
                                }
                                true
                            });

                            // Update enemies
                            let (_dmg, enemy_fire) = enemies.update(dt, player.position);
                            for (origin, dir) in enemy_fire {
                                bullets.fire_enemy(origin, dir);
                            }

                            // Win/lose checks
                            if !player.is_alive() && game.state != GameState::GameOver {
                                let _ = win.set_cursor_grab(CursorGrabMode::None);
                                win.set_cursor_visible(true);
                                cursor_grabbed = false;
                                game.state = GameState::GameOver;
                                game.message = "Game Over! Press ENTER to restart".to_string();
                                game.message_timer = 999.0;
                            }

                            // Wave progression
                            if game.enemies_alive == 0 {
                                if game.current_wave >= 3 {
                                    let _ = win.set_cursor_grab(CursorGrabMode::None);
                                    win.set_cursor_visible(true);
                                    cursor_grabbed = false;
                                    game.state = GameState::Victory;
                                    game.message = format!("Victory! Score: {}  Press ENTER to play again", game.score);
                                    game.message_timer = 999.0;
                                } else {
                                    game.wave_cooldown += dt;
                                    if game.wave_cooldown > 2.0 {
                                        game.current_wave += 1;
                                        let count = enemies_for_wave(game.current_wave);
                                        enemies.clear();
                                        enemies.spawn_for_wave(count);
                                        game.enemies_in_wave = count;
                                        game.enemies_spawned = count;
                                        game.enemies_alive = count;
                                        game.wave_cooldown = 0.0;
                                        game.set_message(format!("Wave {}!", game.current_wave), 2.0);
                                    }
                                }
                            }

                            game.update(dt);
                            rebuild_scene(
                                renderer.device.as_ref().unwrap(),
                                &mut renderer.scene,
                                &items,
                                &enemies,
                                &bullets,
                            );
                        }
                        GameState::Paused | GameState::Menu | GameState::GameOver | GameState::Victory => {
                            game.update(dt);
                        }
                    }

                    renderer.update_scene(dt);
                    let vp = renderer.camera.vp_matrix();
                    let hud = build_hud(&game, renderer.size.0 as f32, renderer.size.1 as f32, vp, &enemies.enemies);
                    renderer.clear_ui();
                    for rect in hud.rects { renderer.add_ui_rect(rect); }
                    if let Some(ref mut text) = renderer.text_renderer {
                        text.clear();
                        for elem in &hud.elements { text.add_text(&elem.text, elem.x, elem.y, elem.scale, elem.color); }
                    }
                    if let Err(ref e) = renderer.render(vp) { log::error!("Render error: {}", e); }
                }
                Event::LoopExiting => {}
                _ => {}
            }
        })
        .expect("Event loop error");
}

fn rebuild_scene(
    device: &wgpu::Device,
    scene: &mut Option<nexforge_renderer::scene::Scene>,
    items: &Items,
    enemies: &Enemies,
    bullets: &Bullets,
) {
    if let Some(ref mut scene) = scene {
        scene.clear_objects();
        build_arena(device, scene);
        build_item_objects(device, scene, items);
        build_enemy_objects(device, scene, enemies);
        build_bullet_objects(device, scene, bullets);
    }
}
