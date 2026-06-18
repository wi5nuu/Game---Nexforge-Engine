#![deny(clippy::all)]

mod settings;

use nexforge_core::{Engine, EngineMode};
use nexscript::InputState;
use settings::parse_env_settings;
use std::cell::RefCell;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

struct DebugInfo {
    fps: f64,
    entity_count: usize,
    frame_count: u64,
    last_fps_update: std::time::Instant,
    fps_frame_count: u64,
}

impl DebugInfo {
    fn new() -> Self {
        Self {
            fps: 0.0,
            entity_count: 0,
            frame_count: 0,
            last_fps_update: std::time::Instant::now(),
            fps_frame_count: 0,
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.fps = 0.0;
        self.entity_count = 0;
        self.frame_count = 0;
        self.last_fps_update = std::time::Instant::now();
        self.fps_frame_count = 0;
    }

    fn update(&mut self, entity_count: usize) {
        self.frame_count += 1;
        self.fps_frame_count += 1;
        self.entity_count = entity_count;
        let elapsed = self.last_fps_update.elapsed();
        if elapsed.as_secs_f64() >= 1.0 {
            self.fps = self.fps_frame_count as f64 / elapsed.as_secs_f64();
            self.fps_frame_count = 0;
            self.last_fps_update = std::time::Instant::now();
        }
    }
}

fn main() {
    env_logger::init();
    let mut settings = parse_env_settings();
    log::info!(
        "Nexforge Game starting with settings: {}x{} {}",
        settings.window_width,
        settings.window_height,
        settings.window_title
    );

    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            log::error!("Failed to create event loop: {}", e);
            return;
        }
    };
    let window = match WindowBuilder::new()
        .with_title(&settings.window_title)
        .with_inner_size(winit::dpi::LogicalSize::new(
            settings.window_width as f64,
            settings.window_height as f64,
        ))
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to create window: {}", e);
            return;
        }
    };

    let mut engine = Engine::new(EngineMode::Game);
    if let Err(e) = engine.initialize() {
        log::error!("Engine init failed: {}", e);
        return;
    }

    engine.audio.set_master_volume(settings.master_volume);

    if let Err(e) = engine.init_renderer(&window) {
        log::warn!("Renderer init failed, running headless: {}", e);
    }

    let script_files = [
        "game/scripts/player.nxs",
        "game/scripts/enemy_ai.nxs",
        "game/scripts/weapon.nxs",
        "game/scripts/game_manager.nxs",
    ];
    let loaded = script_files.iter().filter(|p| engine.load_script(p).is_ok()).count();
    log::info!(
        "Loaded {}/{} scripts, {} entities",
        loaded,
        script_files.len(),
        engine.script.entities.len()
    );

    if let Err(e) = engine.start() {
        log::error!("Engine start failed: {}", e);
        return;
    }

    let mut input = InputState::new();
    let mut last_time = std::time::Instant::now();
    let mut debug = DebugInfo::new();
    let center = RefCell::new((settings.window_width as f32 / 2.0, settings.window_height as f32 / 2.0));
    settings.show_debug_overlay = false;

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: window_event, ..
                } => match window_event {
                    WindowEvent::CloseRequested => {
                        engine.shutdown();
                        target.exit();
                    }
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        let pressed = key_event.state == ElementState::Pressed;
                        if let PhysicalKey::Code(code) = key_event.physical_key {
                            match code {
                                KeyCode::F12 => engine.toggle_console(),
                                KeyCode::F1 => engine.toggle_profiler(),
                                KeyCode::Escape => {
                                    engine.shutdown();
                                    target.exit();
                                }
                                KeyCode::KeyW => input.vertical = if pressed { 1.0 } else { 0.0 },
                                KeyCode::KeyS => input.vertical = if pressed { -1.0 } else { 0.0 },
                                KeyCode::KeyA => input.horizontal = if pressed { -1.0 } else { 0.0 },
                                KeyCode::KeyD => input.horizontal = if pressed { 1.0 } else { 0.0 },
                                KeyCode::Space => input.jump = pressed,
                                KeyCode::ShiftLeft => input.sprint = pressed,
                                KeyCode::ControlLeft => input.crouch = pressed,
                                KeyCode::KeyR => input.reload = pressed,
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == winit::event::MouseButton::Left {
                            input.shoot = state == ElementState::Pressed;
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let (cx, cy) = *center.borrow();
                        input.mouse_x = (position.x as f32 - cx) * settings.mouse_sensitivity;
                        input.mouse_y = (position.y as f32 - cy) * settings.mouse_sensitivity;
                    }
                    WindowEvent::Resized(size) => {
                        *center.borrow_mut() = (size.width as f32 / 2.0, size.height as f32 / 2.0);
                        if let Some(ref mut renderer) = engine.renderer {
                            renderer.resize((size.width, size.height));
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f64().min(0.05);
                    last_time = now;

                    engine.handle_input(&input);
                    engine.tick(dt);

                    if let Some(ref mut renderer) = engine.renderer {
                        renderer.camera.update_mouse(input.mouse_x, input.mouse_y);
                        renderer
                            .camera
                            .update_keyboard(input.horizontal, input.vertical, input.sprint);
                        renderer.update_scene(dt);
                        if let Err(ref e) = renderer.render(renderer.camera.vp_matrix()) {
                            log::error!("Render error at frame {}: {}", engine.frame_count, e);
                        }
                    }

                    debug.update(engine.world.entity_count());
                }
                Event::LoopExiting => {
                    log::info!("Exiting after {} frames", debug.frame_count);
                    engine.shutdown();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}
