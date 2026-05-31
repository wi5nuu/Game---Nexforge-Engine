# Nexforge Engine

![CI](https://github.com/nexforge/nexforge-engine/actions/workflows/ci.yml/badge.svg)
![Release](https://github.com/nexforge/nexforge-engine/actions/workflows/release.yml/badge.svg)
![Rust](https://img.shields.io/badge/rust-1.75.0+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**A blazing-fast FPS/TPS engine built in Rust — from scratch.**

Nexforge Engine is a cutting-edge 3D game engine designed for first and third-person shooters, built entirely in Rust. It features a custom scripting language (NexScript), WebGPU-based rendering pipeline, archetype-based ECS, physics, AI, and rollback netcode.

## Features

- **NexScript DSL** — Custom scripting language with bytecode VM, hot-reload, and coroutines
- **WebGPU Renderer** — Deferred PBR rendering, cascaded shadow maps, post-processing (bloom, SSAO, tone mapping)
- **ECS Core** — Archetype-based, cache-friendly, parallel query O(1) for 100k+ entities
- **Physics** — Rapier3d integration with custom BVH broad-phase
- **Spatial Audio** — HRTF 3D audio with bus system
- **AI System** — Behavior trees, A* NavMesh pathfinding, utility AI
- **Rollback Netcode** — GGPO-inspired deterministic multiplayer
- **Multi-platform** — Windows, macOS, Linux, Web (WASM), Android, iOS

## Build Instructions

### Prerequisites

- Rust 1.75.0 or later (`rustup update stable`)
- Cargo

### Build & Run

```bash
# Build the entire workspace
cargo build --workspace --release

# Run the game
cargo run --release --package nexforge-game

# Run tests
cargo test --workspace

# Run benchmarks
cargo bench --workspace
```

### WebAssembly Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

## Project Structure

```
nexforge-engine/
├── crates/
│   ├── nexscript/          ← NexScript language (lexer, parser, VM)
│   ├── nexforge-ecs/       ← Entity Component System
│   ├── nexforge-renderer/  ← wgpu rendering pipeline
│   ├── nexforge-physics/   ← Physics engine
│   ├── nexforge-audio/     ← Spatial audio
│   ├── nexforge-ai/        ← AI & pathfinding
│   ├── nexforge-net/       ← Rollback netcode
│   └── nexforge-core/      ← Engine entry point
├── game/                   ← Game content
│   ├── scripts/            ← Game logic in NexScript
│   └── assets/             ← Models, textures, audio
├── tools/                  ← Dev console, profiler, LSP
└── docs/                   ← Specifications & architecture
```

## License

MIT
