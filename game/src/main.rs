#![deny(clippy::all)]

use nexforge_core::{Engine, EngineMode};
use nexforge_renderer::camera::Camera;
use nexscript::InputState;
use winit::event::{Event, WindowEvent, ElementState};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

fn main() {
    env_logger::init();
    log::info!("Nexforge Game starting...");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("Nexforge Engine")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .expect("Failed to create window");

    let mut engine = Engine::new(EngineMode::Game);
    if let Err(e) = engine.initialize() {
        log::error!("Engine init failed: {}", e);
        return;
    }
    if let Err(e) = engine.init_renderer(&window) {
        log::warn!("Renderer init failed (headless mode): {}", e);
    }

    let script_files = vec![
        "game/scripts/player.nxs",
        "game/scripts/enemy_ai.nxs",
        "game/scripts/weapon.nxs",
        "game/scripts/game_manager.nxs",
    ];
    for path in &script_files {
        if let Err(e) = engine.load_script(path) {
            log::warn!("Script load failed: {} — {}", path, e);
        }
    }
    log::info!("Loaded {} entity definitions", engine.script.entities.len());

    if let Err(e) = engine.start() {
        log::error!("Engine start failed: {}", e);
        return;
    }

    let mut input = InputState::new();
    let mut last_time = std::time::Instant::now();

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event: window_event, .. } => match window_event {
                WindowEvent::CloseRequested => {
                    engine.shutdown();
                    target.exit();
                }
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    let pressed = key_event.state == ElementState::Pressed;
                    if let PhysicalKey::Code(code) = key_event.physical_key {
                        match code {
                            KeyCode::F12 => { engine.toggle_console(); }
                            KeyCode::F1 => { engine.toggle_profiler(); }
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
                    let center_x = 1280.0 / 2.0;
                    let center_y = 720.0 / 2.0;
                    input.mouse_x = (position.x as f32 - center_x as f32) * 0.002;
                    input.mouse_y = (position.y as f32 - center_y as f32) * 0.002;
                }
                WindowEvent::Resized(size) => {
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
                    renderer.camera.update_keyboard(input.horizontal, input.vertical, input.sprint);
                    let vp = renderer.camera.vp_matrix();
                    if let Err(e) = renderer.render(vp) {
                        log::error!("Render error: {}", e);
                    }
                }
            }
            Event::LoopExiting => {
                engine.shutdown();
            }
            _ => {}
        }
    }).expect("Event loop error");
}
