# Nexforge Engine Architecture

**Version:** 0.1.0 (Draft)

---

## 1. High-Level Architecture

```mermaid
graph TB
    subgraph "Application Layer"
        GAME["Game (NexScript)"]
        UI["UI System"]
        CONSOLE["Dev Console"]
        PROFILER["Frame Profiler"]
    end

    subgraph "Engine Layer"
        CORE["nexforge-core<br/>Engine Entry Point"]
        ECS["nexforge-ecs<br/>Entity Component System"]
        RENDER["nexforge-renderer<br/>wgpu Pipeline"]
        PHYSICS["nexforge-physics<br/>Rapier3d + BVH"]
        AUDIO["nexforge-audio<br/>Spatial Audio"]
        AI["nexforge-ai<br/>Behavior Tree + A*"]
        NET["nexforge-net<br/>Rollback Netcode"]
    end

    subgraph "Scripting Layer"
        VM["nexscript VM<br/>Bytecode Interpreter"]
        COMPILER["nexscript Compiler<br/>Lexer → Parser → Bytecode"]
        HOTRELOAD["Hot-Reload Watcher"]
    end

    subgraph "Platform Layer"
        WINIT["winit — Window/Input"]
        WGPU["wgpu — GPU API"]
        CPAL["cpal — Audio API"]
        RAPIER["rapier3d — Physics"]
    end

    GAME --> NEXSCRIPT["NexScript (.nxs)"]
    NEXSCRIPT --> COMPILER
    COMPILER --> VM
    VM --> ECS
    VM --> RENDER
    VM --> PHYSICS
    VM --> AUDIO
    VM --> AI
    VM --> NET

    CORE --> ECS
    CORE --> RENDER
    CORE --> PHYSICS
    CORE --> AUDIO
    CORE --> AI
    CORE --> NET
    CORE --> VM
    CORE --> HOTRELOAD

    RENDER --> WGPU
    RENDER --> WINIT
    PHYSICS --> RAPIER
    AUDIO --> CPAL

    ECS --> PHYSICS
    ECS --> AI
    ECS --> NET
```

---

## 2. NexScript Compiler & VM Pipeline

```mermaid
flowchart LR
    A["Source (.nxs)"] --> B["Lexer"]
    B --> C["Token Stream"]
    C --> D["Parser"]
    D --> E["AST"]
    E --> F["Bytecode Compiler"]
    F --> G["Bytecode (.nxsb)"]
    G --> H["Virtual Machine"]
    H --> I["Execution"]
    H --> J["Hot-Reload"]
    J --> A
```

---

## 3. Rendering Pipeline

```mermaid
graph TB
    subgraph "Deferred Rendering Pipeline"
        GBUFFER["Geometry Pass<br/>Position, Normal, Albedo, PBR"]
        LIGHTING["Lighting Pass<br/>PBR Shading"]
        SHADOW["Shadow Pass<br/>Cascaded Shadow Maps"]
        POST["Post-Processing<br/>Bloom → SSAO → Tone Mapping"]
        FINAL["Final Output"]
    end

    subgraph "GPU Resources"
        MESHES["Mesh Buffer"]
        MATERIALS["Material Buffer"]
        TEXTURES["Texture Array"]
        LIGHT_DATA["Light Buffer"]
    end

    MESHES --> GBUFFER
    MATERIALS --> GBUFFER
    TEXTURES --> GBUFFER
    LIGHT_DATA --> LIGHTING
    LIGHT_DATA --> SHADOW
    GBUFFER --> LIGHTING
    SHADOW --> LIGHTING
    LIGHTING --> POST
    POST --> FINAL
```

---

## 4. ECS Architecture

```mermaid
graph TB
    subgraph "ECS Core"
        WORLD["World"]
        ARCHETYPE["Archetype Storage"]
        ENTITY["Entity Manager<br/>Generational Index"]
        QUERY["Query System"]
        SCHEDULER["System Scheduler"]
        COMMANDS["Command Buffer"]
    end

    WORLD --> ARCHETYPE
    WORLD --> ENTITY
    WORLD --> COMMANDS
    QUERY --> ARCHETYPE
    QUERY --> SCHEDULER
    SCHEDULER --> SYSTEMS["ECS Systems"]
```

### 4.1 Archetype Layout

```
Archetype: (Transform, RigidBody, Health)
┌────────┬────────────┬──────────┬──────┐
│ Entity │ Transform  │ RigidBody│Health│
├────────┼────────────┼──────────┼──────┤
│    0   │ T0         │ RB0      │ H0   │
│    1   │ T1         │ RB1      │ H1   │
│    2   │ T2         │ RB2      │ H2   │
│  ...   │ ...        │ ...      │ ...  │
└────────┴────────────┴──────────┴──────┘
```

Columns are contiguous arrays for cache-friendly iteration.

---

## 5. Networking Model (Rollback)

```mermaid
sequenceDiagram
    participant ClientA
    participant Server
    participant ClientB

    ClientA->>Server: Input Frame N
    ClientB->>Server: Input Frame N
    Server->>Server: Simulate Frame N
    Server->>ClientA: World State N (diff)
    Server->>ClientB: World State N (diff)

    Note over ClientA,ClientB: Local prediction continues

    Server->>ClientA: World State N+3 (rollback)
    ClientA->>ClientA: Rollback to N, re-simulate with correct inputs
    ClientA->>ClientA: Confirm N+3 state

    Server->>ClientB: World State N+3 (rollback)
    ClientB->>ClientB: Rollback to N, re-simulate
```

---

## 6. Module Dependency Graph

```mermaid
graph LR
    NX["nexscript"] --> ECS["nexforge-ecs"]
    ECS --> RENDER["nexforge-renderer"]
    ECS --> PHYSICS["nexforge-physics"]
    ECS --> AI["nexforge-ai"]
    ECS --> NET["nexforge-net"]
    RENDER --> CORE["nexforge-core"]
    PHYSICS --> CORE
    AUDIO["nexforge-audio"] --> CORE
    AI --> CORE
    NET --> CORE
    NX --> CORE
    CORE --> GAME["game (nexforge-game)"]
```

All crates depend on `nexforge-ecs` for component definitions. The game binary depends on all engine crates via `nexforge-core`.

---

## 7. Data Flow Per Frame

```mermaid
flowchart TD
    A["Input (winit)"] --> B["NexScript VM<br/>Run scripts"]
    B --> C["ECS Systems<br/>Game logic"]
    C --> D["Physics Step<br/>Rapier3d"]
    D --> E["AI Update<br/>Behavior Trees"]
    E --> F["Audio Mix<br/>cpal"]
    F --> G["Render Frame<br/>wgpu"]
    G --> H["Swapchain Present"]
    H --> A
```

**Performance Budget Per Frame (16.6ms total):**

| Stage          | Budget    |
|----------------|-----------|
| NexScript VM   | < 0.1ms  |
| ECS Systems    | < 1ms    |
| Physics        | < 4ms    |
| AI             | < 1ms    |
| Audio          | < 1ms    |
| Render         | < 8ms    |
| Buffer         | ~1.5ms   |

---

## 8. Platform Abstraction

```mermaid
graph TB
    subgraph "Platform Layer"
        PC["Desktop (Windows/macOS/Linux)"]
        WEB["Web (WASM)"]
        MOBILE["Mobile (Android/iOS)"]
    end

    subgraph "Abstraction"
        WINIT_LAYER["winit Window"]
        WGPU_LAYER["wgpu Graphics"]
        CPAL_LAYER["cpal Audio"]
        FILE_LAYER["File I/O"]
    end

    PC --> WINIT_LAYER
    PC --> WGPU_LAYER
    PC --> CPAL_LAYER
    PC --> FILE_LAYER

    WEB --> WGPU_LAYER
    WEB --> FILE_LAYER

    MOBILE --> WINIT_LAYER
    MOBILE --> WGPU_LAYER
    MOBILE --> CPAL_LAYER
    MOBILE --> FILE_LAYER
```

---

## 9. Dev Console Architecture

```mermaid
graph LR
    INPUT["F12 Toggle"] --> CMD["Command Processor"]
    CMD --> PARSER["Arg Parser"]
    PARSER --> HANDLER["Command Handlers"]
    HANDLER --> ENTITIES["Entity Inspector"]
    HANDLER --> STATS["Performance Stats"]
    HANDLER --> ENV["Engine Environment"]
```

Commands: `spawn`, `set_gravity`, `teleport`, `god_mode`, `list_entities`, `set_time_scale`.

---

## 10. Profiler Architecture

```mermaid
graph TB
    subgraph "Profiler"
        TIMERS["System Timers"]
        GPU_QUERIES["wgpu Timestamp Queries"]
        MEM_TRACKER["Memory Tracker"]
        JSON_EXPORT["JSON Export"]
    end

    subgraph "Display"
        OVERLAY["F1 Overlay"]
        GRAPH["Frame Time Graph"]
        TABLE["System Breakdown Table"]
    end

    TIMERS --> OVERLAY
    TIMERS --> GRAPH
    TIMERS --> TABLE
    GPU_QUERIES --> TABLE
    GPU_QUERIES --> JSON_EXPORT
    MEM_TRACKER --> TABLE
```

---

## 11. File Layout

```
nexforge-engine/
├── .github/workflows/       ← CI/CD pipelines
├── crates/
│   ├── nexscript/           ← Lexer, Parser, AST, Bytecode Compiler, VM
│   ├── nexforge-ecs/        ← World, Entity, Component, Query, Scheduler
│   ├── nexforge-renderer/   ← Pipeline, PBR, Shadow, Post-process, Shaders
│   ├── nexforge-physics/    ← Rapier3d integration, BVH, Character Controller
│   ├── nexforge-audio/      ← cpal output, HRTF, Audio Bus
│   ├── nexforge-ai/         ← Behavior Tree, A*, NavMesh, Utility AI
│   ├── nexforge-net/        ← Rollback, Deterministic Sim, WebRTC/UDP
│   └── nexforge-core/       ← Engine entry, System scheduler, Platform layer
├── game/
│   ├── src/                 ← Game binary (main.rs)
│   ├── scripts/             ← NexScript game logic files
│   └── assets/              ← Models, textures, audio
├── tools/
│   ├── dev-console/         ← In-game dev console tooling
│   ├── profiler/            ← Frame profiler
│   └── nexscript-lsp/       ← Language Server Protocol for NexScript
└── docs/                    ← Specifications and architecture docs
```

---

## 12. Key Design Decisions

| Decision                    | Choice                    | Rationale                                 |
|-----------------------------|---------------------------|-------------------------------------------|
| Graphics API                | WebGPU (wgpu)             | Cross-platform, future-proof, safe        |
| ECS Storage                 | Archetype-based           | Cache-friendly, O(1) query                |
| Physics                     | Rapier3d + Custom BVH     | Battle-tested foundation + custom perf    |
| Scripting                   | Custom bytecode VM        | Deterministic, embeddable, hot-reloadable |
| Networking                  | Rollback netcode          | GGPO-style for responsive multiplayer     |
| Audio                       | cpal + HRTF               | Cross-platform, low-latency               |
| Windowing                   | winit                     | Pure Rust, cross-platform                 |
| Serialization               | bincode + serde           | Compact binary, fast                      |
| Error Handling              | thiserror                 | Idiomatic Rust error types                |
| Build System                | Cargo workspace           | Native Rust, fast incremental builds      |
