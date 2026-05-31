# NexScript Language Specification

**Version:** 0.1.0 (Preview)
**Status:** Draft

---

## 1. Philosophy

NexScript is a statically-typed, compiled scripting language purpose-built for the Nexforge Engine. It is designed to empower game designers to write gameplay logic without touching Rust, while providing safety, performance, and hot-reload capability.

**Core Principles:**
- **Static typing with inference** — Catch errors at compile time, not runtime
- **No garbage collector** — Arena-based allocation, predictable performance
- **Hot-reload** — Edit scripts while the game runs, preserve entity state
- **Deterministic** — Same script + same input = same output (critical for rollback netcode)
- **Game-first** — First-class support for ECS, vectors, entities, and components

---

## 2. Syntax & Types

### 2.1 Comments

```nexscript
// Single-line comment
/* Multi-line
   comment */
```

### 2.2 Primitive Types

| Type      | Description        | Size    | Example        |
|-----------|--------------------|---------|----------------|
| `int`     | Signed 32-bit int  | 4 bytes | `42`, `-7`     |
| `float`   | 32-bit float       | 4 bytes | `3.14`, `-1.0` |
| `bool`    | Boolean            | 1 byte  | `true`, `false`|
| `string`  | UTF-8 string       | dynamic | `"hello"`      |
| `void`    | No value           | 0       | —              |

### 2.3 Vector Types

| Type         | Description       | Example               |
|--------------|-------------------|-----------------------|
| `vec2`       | 2D float vector   | `vec2(1.0, 2.0)`     |
| `vec3`       | 3D float vector   | `vec3(1.0, 2.0, 3.0)`|
| `vec4`       | 4D float vector   | `vec4(1.0, 2.0, 3.0, 1.0)`|
| `quat`       | Quaternion        | `quat(0.0, 0.0, 0.0, 1.0)`|

### 2.4 Game Types

| Type         | Description                    |
|--------------|--------------------------------|
| `entity`     | Reference to an ECS entity     |
| `component`  | Reference to a component type  |
| `Transform`  | Position, rotation, scale      |
| `RigidBody`  | Physics body                   |
| `Camera`     | Camera with FOV, near/far      |
| `Health`     | Health with max/current        |
| `Weapon`     | Weapon component               |

### 2.5 Variable Declarations

```nexscript
let x: int = 42          // Explicit type
let y = 3.14             // Type inferred as float
let name = "player"      // Type inferred as string
let mut health = 100     // Mutable variable
health = 80              // Mutation allowed
```

### 2.6 Functions

```nexscript
fn add(a: int, b: int) -> int {
    return a + b
}

fn greet(name: string) -> void {
    log("Hello, " + name)
}

// Function with default parameter
fn spawn_enemy(pos: vec3, health: float = 100.0) -> entity {
    // ...
}
```

### 2.7 Control Flow

```nexscript
// If-else
if health <= 0 {
    die()
} else if health < 20 {
    flee()
} else {
    attack()
}

// While loop
let mut i = 0
while i < 10 {
    spawn_enemy(vec3(i as float, 0, 0))
    i = i + 1
}

// For loop (range)
for i in 0..10 {
    log("Iteration: " + i.to_string())
}

// For loop (collection)
for e in entities_with<Enemy> {
    e.Health.damage(10)
}
```

### 2.8 Operators

| Category       | Operators                          |
|----------------|------------------------------------|
| Arithmetic     | `+`, `-`, `*`, `/`, `%`           |
| Comparison     | `==`, `!=`, `<`, `>`, `<=`, `>=`  |
| Logical        | `&&`, `||`, `!`                   |
| Vector         | `.dot()`, `.cross()`, `.normalize()`, `.length()` |
| Assignment     | `=`, `+=`, `-=`, `*=`, `/=`       |

---

## 3. Entity & Component System

### 3.1 Entity Definition

```nexscript
entity Player {
    component Transform
    component RigidBody
    component Camera { fov: 90.0 }
    component Health { max: 100, current: 100 }
    component Weapon
}
```

### 3.2 Component Declaration

```nexscript
component Health {
    max: int,
    current: int,

    fn damage(amount: int) {
        self.current = max(self.current - amount, 0)
    }

    fn is_alive() -> bool {
        return self.current > 0
    }
}
```

### 3.3 Component Access

```nexscript
// Within an entity block
self.Health.current
self.Transform.position
self.RigidBody.set_velocity(vec3(0, 0, 10))

// External access
let player = find_entity("player_1")
player.Health.damage(25)
player.Transform.position.y = 5.0
```

---

## 4. Event System

### 4.1 Built-in Events

```nexscript
entity Player {
    // Called every frame
    on_update(dt: float) {
        // Movement, input handling
    }

    // Called on physics collision
    on_collision(other: entity, contact: ContactInfo) {
        // Collision response
    }

    // Called when entity is spawned
    on_spawn() {
        // Initialization
    }

    // Called when entity dies (health <= 0)
    on_death() {
        // Death effects, respawn
    }

    // Called when entity enters a trigger zone
    on_trigger_enter(other: entity) {
        // Trigger logic
    }

    // Called when entity leaves a trigger zone
    on_trigger_exit(other: entity) {
        // Exit logic
    }
}
```

### 4.2 Custom Events

```nexscript
event OnPickup(player: entity, item: string)

entity Pickup {
    component Transform
    item_type: string

    on_trigger_enter(other: entity) {
        if other.has<Player> {
            emit OnPickup(other, self.item_type)
            destroy(self)
        }
    }
}
```

---

## 5. Built-in Functions & API

### 5.1 Math

| Function                            | Description            |
|-------------------------------------|------------------------|
| `sin(x: float) -> float`            | Sine                   |
| `cos(x: float) -> float`            | Cosine                 |
| `tan(x: float) -> float`            | Tangent                |
| `sqrt(x: float) -> float`           | Square root            |
| `abs(x: float) -> float`            | Absolute value         |
| `clamp(v, min, max)`               | Clamp value            |
| `lerp(a, b, t)`                     | Linear interpolation   |
| `random() -> float`                | Random [0.0, 1.0)     |
| `random_range(min, max) -> float`  | Random in range        |

### 5.2 Input

```nexscript
let input = Input.get()
// input.horizontal: float  — Left/Right (-1 to 1)
// input.vertical: float    — Forward/Back (-1 to 1)
// input.mouse_x: float     — Mouse delta X
// input.mouse_y: float     — Mouse delta Y
// input.jump: bool         — Jump pressed
// input.shoot: bool        — Shoot pressed
// input.reload: bool       — Reload pressed
// input.sprint: bool       — Sprint held
// input.crouch: bool       — Crouch held
```

### 5.3 ECS Queries

```nexscript
// Find all entities with specific components
for e in entities_with<Enemy, Health> {
    if e.Health.is_alive() {
        e.AI.set_state("chase")
    }
}

// Find single entity
let player = find_entity("player")
let nearest = find_nearest(pos, entities_with<Pickup>)
```

### 5.4 Physics

```nexscript
// Raycasting
let hit = raycast(origin, direction, max_distance)
if hit.is_some() {
    log("Hit: " + hit.entity.to_string())
}

// Trigger zones
trigger_zone("win_area", vec3(0, 0, 0), 5.0)
```

### 5.5 Audio

```nexscript
play_sound("sfx/shoot.wav")
play_sound_3d("sfx/explosion.wav", position)
set_music("music/combat.ogg")
set_volume("sfx", 0.8)
```

### 5.6 UI

```nexscript
show_ui("health_bar")
hide_ui("scoreboard")
update_text("ammo_counter", "12 / 30")
```

### 5.7 Debug

```nexscript
log("Player health: " + health.to_string())
draw_line(start, end, color)
draw_sphere(center, radius, color)
```

---

## 6. Coroutines & Async

```nexscript
// Coroutine using yield
coroutine fn spawn_wave(count: int, interval: float) {
    for i in 0..count {
        spawn_enemy(choose_spawn_point())
        yield(interval)  // Pause for interval seconds
    }
    log("Wave complete!")
}

// Await a coroutine
on_update(dt: float) {
    if input.spawn_wave {
        await spawn_wave(10, 0.5)
    }
}
```

---

## 7. Script Examples

### 7.1 Player Controller

```nexscript
entity Player {
    component Transform
    component RigidBody
    component Camera { fov: 90.0 }
    component Health { max: 100, current: 100 }
    component Weapon

    on_update(dt: float) {
        let input = Input.get()

        // Movement
        let move_dir = vec3(input.horizontal, 0, input.vertical)
        if move_dir.length() > 0 {
            move_dir = move_dir.normalize()
            let speed = if input.sprint { 9.0 } else { 6.0 }
            self.RigidBody.set_velocity(move_dir * speed)
        }

        // Camera look
        self.Camera.yaw += input.mouse_x * 0.002
        self.Camera.pitch = clamp(
            self.Camera.pitch + input.mouse_y * 0.002,
            -1.5, 1.5
        )

        // Jump
        if input.jump && self.RigidBody.is_grounded() {
            self.RigidBody.apply_impulse(vec3(0, 8, 0))
        }

        // Shoot
        if input.shoot {
            self.Weapon.fire()
        }

        // Reload
        if input.reload {
            self.Weapon.reload()
        }

        // Crouch
        if input.crouch {
            self.Transform.scale.y = 0.5
        } else {
            self.Transform.scale.y = 1.0
        }
    }

    on_death() {
        play_sound("sfx/player_death.wav")
        show_ui("death_screen")
        GameManager.trigger_respawn(self, 3.0)
    }
}
```

### 7.2 Enemy AI

```nexscript
entity Enemy {
    component Transform
    component RigidBody
    component Health { max: 50, current: 50 }
    component AIState { current: "patrol" }

    state patrol_timer: float = 0.0
    state patrol_target: vec3 = vec3(0, 0, 0)
    state alert_radius: float = 15.0

    on_update(dt: float) {
        let player = find_entity("player")
        if player == null { return }

        let dist = (self.Transform.position - player.Transform.position).length()

        if self.AIState.current == "patrol" {
            handle_patrol(dt, player, dist)
        } else if self.AIState.current == "alert" {
            handle_alert(dt, player, dist)
        } else if self.AIState.current == "attack" {
            handle_attack(dt, player, dist)
        }
    }

    fn handle_patrol(dt: float, player: entity, dist: float) {
        self.patrol_timer -= dt
        if self.patrol_timer <= 0 {
            self.patrol_target = random_navmesh_point()
            self.patrol_timer = 4.0
        }
        move_towards(self.patrol_target, 3.0)

        if dist < self.alert_radius {
            self.AIState.current = "alert"
        }
    }

    fn handle_alert(dt: float, player: entity, dist: float) {
        move_towards(player.Transform.position, 4.5)

        if dist < 8.0 {
            self.AIState.current = "attack"
        } else if dist > 20.0 {
            self.AIState.current = "patrol"
        }
    }

    fn handle_attack(dt: float, player: entity, dist: float) {
        // Seek cover
        let cover = find_nearest_cover(self.Transform.position, player.Transform.position)
        if cover != null {
            move_towards(cover, 5.0)
        }

        // Aim and shoot
        self.Transform.look_at(player.Transform.position)

        if dist < 15.0 {
            // Fire weapon with spread
            let spread = 0.05
            let dir = (player.Transform.position - self.Transform.position).normalize()
            dir += vec3(random_range(-spread, spread), random_range(-spread, spread), random_range(-spread, spread))
            fire_projectile(self.Transform.position, dir, 20.0)
        }

        if dist > 25.0 {
            self.AIState.current = "alert"
        }
    }

    on_death() {
        play_sound("sfx/enemy_death.wav")
        spawn_loot(self.Transform.position)
        GameManager.add_score(100)
    }
}
```

### 7.3 Weapon System

```nexscript
component Weapon {
    ammo: int,
    max_ammo: int,
    fire_rate: float,
    reload_time: float,
    damage: float,
    spread: float,
    recoil_pattern: [vec2],

    state cooldown: float = 0.0
    state is_reloading: bool = false
    state recoil_index: int = 0

    on_init() {
        self.ammo = self.max_ammo
        self.cooldown = 0.0
    }

    fn fire() {
        if self.is_reloading { return }
        if self.cooldown > 0 { return }
        if self.ammo <= 0 {
            self.reload()
            return
        }

        self.ammo -= 1
        self.cooldown = 1.0 / self.fire_rate

        // Apply recoil
        if self.recoil_index < self.recoil_pattern.length() {
            let recoil = self.recoil_pattern[self.recoil_index]
            apply_recoil(recoil.x, recoil.y)
            self.recoil_index += 1
        }

        // Raycast shoot
        let cam = self.entity.Camera
        let spread_vec = vec3(
            random_range(-self.spread, self.spread),
            random_range(-self.spread, self.spread),
            random_range(-self.spread, self.spread)
        )
        let direction = cam.forward() + spread_vec
        let hit = raycast(cam.position(), direction.normalize(), 100.0)

        if hit.is_some() {
            let target = hit.entity
            if target.has<Health> {
                target.Health.damage(self.damage)
                spawn_hit_effect(hit.position)
                play_sound("sfx/hit.wav")
            }
        }

        play_sound("sfx/shoot.wav")
        spawn_muzzle_flash()
    }

    fn reload() {
        if self.is_reloading { return }
        if self.ammo == self.max_ammo { return }

        self.is_reloading = true
        await wait(self.reload_time)
        self.ammo = self.max_ammo
        self.is_reloading = false
        self.recoil_index = 0
    }

    on_update(dt: float) {
        if self.cooldown > 0 {
            self.cooldown -= dt
        }
    }
}
```

### 7.4 Game Manager

```nexscript
entity GameManager {
    component Transform

    state score: int = 0
    state round: int = 1
    state enemies_alive: int = 0
    state max_enemies: int = 10
    state is_game_over: bool = false

    on_spawn() {
        log("Game started!")
        start_round()
    }

    fn start_round() {
        log("Round " + self.round.to_string() + " starting!")
        self.max_enemies = 5 + (self.round - 1) * 2
        self.enemies_alive = self.max_enemies

        for i in 0..self.max_enemies {
            let spawn_point = choose_spawn_point()
            spawn_enemy(spawn_point)
        }
    }

    fn add_score(points: int) {
        self.score += points
        self.enemies_alive -= 1
        update_text("score_text", "Score: " + self.score.to_string())
        update_text("enemy_count", "Enemies: " + self.enemies_alive.to_string())

        if self.enemies_alive <= 0 {
            self.round += 1
            start_round()
        }
    }

    fn trigger_respawn(player: entity, delay: float) {
        await wait(delay)
        let spawn_point = find_spawn_point("player")
        player.Transform.position = spawn_point
        player.Health.current = player.Health.max
        hide_ui("death_screen")
    }

    on_update(dt: float) {
        if self.is_game_over { return }

        // Check win/lose conditions
        let all_players = entities_with<Player, Health>
        let players_alive = 0
        for p in all_players {
            if p.Health.is_alive() {
                players_alive += 1
            }
        }

        if players_alive <= 0 {
            self.is_game_over = true
            show_ui("game_over_screen")
            log("Game Over! Final score: " + self.score.to_string())
        }
    }
}
```

---

## 8. Hot-Reload Protocol

NexScript supports hot-reloading without engine restart:

1. The file watcher detects `.nxs` file changes
2. The script is recompiled to bytecode
3. Entity state is preserved (components are kept)
4. Old VM instance is swapped atomically
5. `on_spawn()` is NOT re-called (state preserved)
6. Errors are reported with file and line number

**State Preservation Rules:**
- Component data persists across reloads
- `state` variables persist
- New functions are available immediately
- Removed functions cause compile-time errors
- Changed component schemas require entity migration

---

## 9. Error Handling

```nexscript
// Runtime errors are caught and reported
let result = try risky_operation()
if result.is_error() {
    log("Error: " + result.error_message())
}

// Panic stops the script and reports
panic("Something went wrong at " + __line__.to_string())
```

**Error Message Format:**
```
Error [player.nxs:42]: Type mismatch: expected 'float', got 'int'
Error [enemy_ai.nxs:78]: Undefined function 'move_towrads' (did you mean 'move_towards'?)
```

---

## 10. Limitations

- No dynamic memory allocation at runtime (arena-based)
- No closures or anonymous functions
- No operator overloading
- Single-threaded per script instance (ECS systems are multi-threaded)
- No runtime reflection
- No garbage collection — cycles will leak (use weak references)

---

## 11. Grammar (EBNF)

```ebnf
program        = { statement | entity_def | component_def | event_def | fn_def }

statement      = var_decl | assignment | if_stmt | while_stmt | for_stmt
               | return_stmt | expr_stmt | block | break_stmt

var_decl       = "let" ["mut"] ident [":" type] "=" expr ";"
assignment     = lvalue ("=" | "+=" | "-=" | "*=" | "/=") expr ";"
if_stmt        = "if" expr block ["else" (if_stmt | block)]
while_stmt     = "while" expr block
for_stmt       = "for" ident "in" expr ".." expr block
              | "for" ident "in" entities_with "<" ident ">" block
return_stmt    = "return" [expr] ";"
break_stmt     = "break" ";"

entity_def     = "entity" ident "{" { component_ref } { event_handler } "}"
component_ref  = "component" ident ["{" { ident ":" type "," } "}"]
component_def  = "component" ident "{" { field_decl } { fn_def } "}"
event_def      = "event" ident "(" [params] ")" ";"
event_handler  = "on_" ident "(" [params] ")" block

fn_def         = ["coroutine"] "fn" ident "(" [params] ")" ["->" type] block
block          = "{" { statement } "}"
expr           = literal | ident | binary_op | unary_op | call | member_access
               | "if" expr block "else" block

type           = "int" | "float" | "bool" | "string" | "void"
               | "vec2" | "vec3" | "vec4" | "quat"
               | "entity" | "component"
               | ident
```

---

## 12. Bytecode Instruction Set

| Opcode           | Operands        | Description                   |
|------------------|-----------------|-------------------------------|
| `NOP`            | —               | No operation                  |
| `PUSH_INT`       | i32             | Push integer constant         |
| `PUSH_FLOAT`     | f32             | Push float constant           |
| `PUSH_BOOL`      | bool            | Push boolean constant         |
| `PUSH_STRING`    | string index    | Push string constant          |
| `PUSH_VEC3`      | f32, f32, f32   | Push vec3 constant            |
| `PUSH_NULL`      | —               | Push null                     |
| `POP`            | —               | Pop value                     |
| `DUP`            | —               | Duplicate top                 |
| `LOAD_LOCAL`     | u8              | Load local variable           |
| `STORE_LOCAL`    | u8              | Store local variable          |
| `LOAD_FIELD`     | u16             | Load component field          |
| `STORE_FIELD`    | u16             | Store component field         |
| `ADD`            | —               | Add                           |
| `SUB`            | —               | Subtract                      |
| `MUL`            | —               | Multiply                      |
| `DIV`            | —               | Divide                        |
| `NEG`            | —               | Negate                        |
| `EQ`             | —               | Equal                         |
| `NEQ`            | —               | Not equal                     |
| `LT`             | —               | Less than                     |
| `GT`             | —               | Greater than                  |
| `LE`             | —               | Less or equal                 |
| `GE`             | —               | Greater or equal              |
| `AND`            | —               | Logical AND                   |
| `OR`             | —               | Logical OR                    |
| `NOT`            | —               | Logical NOT                   |
| `JMP`            | u16             | Unconditional jump            |
| `JMP_IF`         | u16             | Jump if true                  |
| `JMP_IF_NOT`     | u16             | Jump if false                 |
| `CALL`           | u16             | Call function                 |
| `CALL_BUILTIN`   | u8              | Call built-in function        |
| `RETURN`         | —               | Return from function          |
| `YIELD`          | —               | Yield coroutine               |
| `AWAIT`          | —               | Await coroutine               |
| `ENTITY_GET`     | —               | Get entity component          |
| `NEW_ENTITY`     | —               | Spawn new entity              |
| `DESTROY`        | —               | Destroy entity                |
| `HALT`           | —               | Stop execution                |
