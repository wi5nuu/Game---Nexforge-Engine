# Nexforge Engine

[![CI](https://github.com/wi5nuu/Game---Nexforge-Engine/actions/workflows/ci.yml/badge.svg)](https://github.com/wi5nuu/Game---Nexforge-Engine/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.75.0+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A 3D game engine built in Rust for first and third-person shooters, featuring a custom scripting language (NexScript), WebGPU rendering pipeline, archetype-based ECS, physics simulation, spatial audio, AI behavior trees, and rollback netcode.

---

## Architecture

```
nexforge-engine/
├── crates/
│   ├── nexscript/             NexScript scripting language (lexer, parser, AST, compiler, VM, runtime, hot-reload)
│   ├── nexforge-ecs/          Archetype-based Entity Component System
│   ├── nexforge-renderer/     wgpu deferred PBR rendering pipeline
│   ├── nexforge-physics/      Physics engine (Rapier3d + custom BVH broad-phase)
│   ├── nexforge-audio/        Spatial 3D audio (HRTF, audio buses)
│   ├── nexforge-ai/           AI system (behavior trees, NavMesh pathfinding, utility AI)
│   ├── nexforge-net/          Rollback netcode (GGPO-inspired deterministic multiplayer)
│   └── nexforge-core/         Engine orchestrator (subsystem integration, game loop)
├── game/
│   ├── src/
│   │   ├── main.rs            Game entry point (winit event loop, input handling, render loop)
│   │   └── systems/           Game systems (player, enemy, weapon, game manager)
│   ├── scripts/               Game logic in NexScript (.nxs files)
│   └── assets/                Models, textures, audio resources
├── tools/
│   ├── dev-console/           In-game developer console (F12)
│   ├── profiler/              Frame profiler (F1, GPU timing, memory tracking, JSON export)
│   └── nexscript-lsp/        Language Server Protocol implementation for NexScript
├── docs/
│   ├── ARCHITECTURE.md        System architecture documentation
│   ├── NEXSCRIPT_SPEC.md      NexScript language specification
│   └── CONTRIBUTING.md        Contribution guidelines
├── Cargo.toml                 Workspace root
├── clippy.toml                Clippy lint configuration
└── .rustfmt.toml              Code formatting configuration
```

## Features

### NexScript Language
- Custom scripting language with 44 token types, 30+ AST node types, and 40 bytecode opcodes
- Stack-based bytecode VM supporting 6 value types (Int, Float, Bool, String, Vec3, Null)
- Coroutines with yield/await support
- Hot-reload: file watching (poll-based, 500ms interval) with automatic recompilation while preserving entity state
- 9 built-in functions (log, sin, cos, sqrt, abs, clamp, random, print, pop)
- Entity definitions, component definitions, events, state variables, for ranges, and standard control flow
- Integration with Rust ECS via bidirectional entity mapping

### ECS Core
- Archetype-based entity storage with columnar component arrays for cache-friendly iteration
- Generational entity handles (u32 index + u32 generation)
- Command buffer for deferred spawn/despawn/component operations
- System scheduler with ordered execution
- Query system for efficient entity iteration
- Component registry with type-erased metadata

### Rendering Pipeline
- WebGPU (wgpu 0.19) with Vulkan/DX12/Metal backends
- Deferred PBR rendering with albedo, metallic, roughness, ambient occlusion, emissive, and normal mapping
- Cascaded shadow maps with configurable cascade count and split lambda
- Post-processing stack: bloom (multi-pass), SSAO, motion blur, tone mapping (ACES, Reinhard, Unreal)
- Custom WGSL shaders for all passes

### Physics Engine
- Rapier3d integration for rigid body dynamics and collision detection
- Custom BVH (Bounding Volume Hierarchy) broad-phase for spatial queries
- Character controller with slope handling, step climbing, and grounding detection
- Trigger zones for area-based gameplay events
- Raycasting against the physics world

### Spatial Audio
- HRTF-based 3D audio positioning (cpal backend)
- 4 audio bus channels: SFX, Music, Voice, Ambient
- Per-source attenuation, doppler pitch shifting, and looping
- Programmatic sine wave audio clip generation

### AI System
- Behavior trees with Sequence, Selector, Parallel, Condition, Action, Inverter, and Repeat nodes
- A* pathfinding on configurable NavMesh with ad-hoc node/edge/triangle graph
- Utility AI with scoring curves (linear, quadratic, inverse, logistic)
- Cover point system for tactical positioning

### Rollback Netcode
- GGPO-inspired deterministic rollback with frame-level state management
- Input buffer (ring buffer with frame tracking) for client-side input accumulation
- Delta compression between world snapshots for bandwidth efficiency
- Client-side prediction with configurable rollback window
- Player input serialization (bincode-based) with WASD, mouse, and action buttons

### Developer Tools
- **Dev Console (F12)**: Command registry with 11 default commands (help, clear, entities, fps, stats, god, spawn, tp, gravity, time, exit), command history, multi-level logging
- **Frame Profiler (F1)**: GPU timestamps, memory tracking, FPS/frame-time/p99 statistics, scoped timers, JSON export
- **NexScript LSP**: Diagnostics (typo detection, missing semicolons), autocompletion (keywords, types, builtins, user functions/entities), hover information, go-to-definition

---

## Game Systems (nexforge-game)

The game binary implements a 3D FPS wave-survival arena shooter:

| System         | Responsibility                                                    |
|----------------|------------------------------------------------------------------|
| Wave Survival  | 3 escalating waves (3/5/7 enemies), 2s cooldown between waves, Victory after wave 3 |
| Combat         | Semi-auto rifle (25 damage), 30-round magazine, 1.5s reload (R key) |
| Enemies        | 3 types: Fast (30 HP, 1.4x speed), Normal (50 HP), Tanky (80 HP, 0.7x speed) |
| Enemy AI       | Patrol 3 waypoints, chase within 8 units, shoot at player when in range |
| Items          | 8 collectibles: gold (+100 score) and green health (+25 HP) with float animation |
| HUD            | HP bar, ammo bar, wave counter, timer, crosshair, enemy health bars, gun view model |
| Screens        | Menu screen (controls/objectives), Pause (ESC), Victory/GameOver with stats |

Controls:
| Key/Input         | Action               |
|-------------------|----------------------|
| W / A / S / D     | Camera-relative move |
| Mouse Move        | Look around          |
| Mouse Left Click  | Shoot                |
| R                 | Reload               |
| ShiftLeft         | Sprint               |
| ENTER             | Start / Restart      |
| ESC               | Pause / Quit to menu |

---

## Build Instructions

### Prerequisites

- Rust 1.75.0 or later (`rustup update stable`)
- Cargo
- Vulkan SDK, DirectX 12, or Metal (depending on platform) -- wgpu handles backend selection automatically

### Build

```bash
# Build entire workspace
cargo build --workspace --release

# Build a specific crate
cargo build -p nexforge-renderer --release
```

### Run

```bash
# Run the game
cargo run --release -p nexforge-game
```

### Test

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p nexforge-ecs
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace
```

### WebAssembly Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

---

## NexScript Language Quick Reference

```rust
// Entity definition with components and event handlers
entity Player {
    component Health { max: 100.0 }

    on_spawn() {
        log("Player spawned");
    }

    on_update(dt) {
        let speed = 6.0;
        // movement logic
    }
}

// Standalone functions
fn add(a, b) {
    return a + b;
}

// Control flow
if x > 10 {
    log("large");
} else {
    log("small");
}

while hp > 0 {
    hp -= 1;
}

for i in 0..10 {
    log(i);
}

// Coroutines
coroutine spawn_wave() {
    yield;
}
```

---

## Workspace Dependencies

| Crate       | Version | Description                                    |
|-------------|---------|------------------------------------------------|
| nexscript   | 0.1.0   | Lexer, parser, bytecode compiler, VM, hot-reload |
| nexforge-ecs| 0.1.0   | Archetype-based Entity Component System        |
| nexforge-renderer| 0.1.0| wgpu rendering pipeline (PBR, shadows, post-proc) |
| nexforge-physics | 0.1.0| Physics engine (Rapier3d, BVH, character controller) |
| nexforge-audio | 0.1.0 | Spatial 3D audio (cpal, HRTF, audio buses)    |
| nexforge-ai | 0.1.0   | Behavior trees, A* NavMesh, utility AI         |
| nexforge-net | 0.1.0  | Rollback netcode (GGPO, delta compression)     |
| nexforge-core | 0.1.0 | Engine entry point and platform abstraction    |
| nexforge-game | 0.1.0 | FPS/TPS game binary                            |
| nexforge-dev-console | 0.1.0 | In-game developer console             |
| nexforge-profiler | 0.1.0 | Frame profiler with GPU timing          |
| nexscript-lsp | 0.1.0 | Language Server Protocol for NexScript         |

---

## License

MIT
