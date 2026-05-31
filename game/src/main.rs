#![deny(clippy::all)]

use nexforge_core::Engine;

fn main() {
    env_logger::init();
    log::info!("Nexforge Engine starting...");

    let mut engine = Engine::new();
    match engine.initialize() {
        Ok(()) => log::info!("Engine initialized successfully."),
        Err(e) => {
            log::error!("Failed to initialize engine: {}", e);
            return;
        }
    }

    log::info!(
        "Engine ready. Frame time: {:.2}ms",
        engine.frame_time() * 1000.0
    );

    if let Err(e) = engine.run() {
        log::error!("Engine runtime error: {}", e);
    }

    engine.shutdown();
    log::info!("Nexforge Engine shutdown complete.");
}
