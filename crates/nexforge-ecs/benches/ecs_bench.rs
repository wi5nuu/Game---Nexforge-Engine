use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nexforge_ecs::component::Component;
use nexforge_ecs::entity::Entity;
use nexforge_ecs::world::World;

struct Position(f32, f32, f32);
struct Velocity(f32, f32, f32);
struct Health(f32);

impl Component for Position {
    fn id() -> u64 { 1 }
    fn size() -> usize { std::mem::size_of::<Self>() }
    fn align() -> usize { std::mem::align_of::<Self>() }
}

impl Component for Velocity {
    fn id() -> u64 { 2 }
    fn size() -> usize { std::mem::size_of::<Self>() }
    fn align() -> usize { std::mem::align_of::<Self>() }
}

impl Component for Health {
    fn id() -> u64 { 3 }
    fn size() -> usize { std::mem::size_of::<Self>() }
    fn align() -> usize { std::mem::align_of::<Self>() }
}

fn bench_spawn_100k(c: &mut Criterion) {
    c.bench_function("ecs_spawn_100k", |b| {
        b.iter(|| {
            let mut world = World::new();
            for i in 0..100_000 {
                let entity = world.spawn();
                world.add_component(entity, Position(i as f32, 0.0, 0.0));
                world.add_component(entity, Velocity(1.0, 0.0, 0.0));
                world.add_component(entity, Health(100.0));
            }
            black_box(world.entity_count());
        })
    });
}

fn bench_query_100k(c: &mut Criterion) {
    c.bench_function("ecs_query_100k", |b| {
        let mut world = World::new();
        for i in 0..100_000 {
            let entity = world.spawn();
            world.add_component(entity, Position(i as f32, 0.0, 0.0));
            world.add_component(entity, Velocity(1.0, 0.0, 0.0));
            world.add_component(entity, Health(100.0));
        }
        b.iter(|| {
            let results = world.query_entities(&[Position::id(), Velocity::id()]);
            black_box(results.len());
        })
    });
}

fn bench_despawn_10k(c: &mut Criterion) {
    c.bench_function("ecs_despawn_10k", |b| {
        b.iter(|| {
            let mut world = World::new();
            let mut entities = Vec::new();
            for i in 0..10_000 {
                let entity = world.spawn();
                world.add_component(entity, Position(i as f32, 0.0, 0.0));
                world.add_component(entity, Health(100.0));
                entities.push(entity);
            }
            for entity in &entities {
                world.despawn(*entity);
            }
            black_box(world.entity_count());
        })
    });
}

fn bench_add_remove_component(c: &mut Criterion) {
    c.bench_function("ecs_add_remove_10k", |b| {
        b.iter(|| {
            let mut world = World::new();
            let mut entities = Vec::new();
            for i in 0..10_000 {
                let entity = world.spawn();
                world.add_component(entity, Position(i as f32, 0.0, 0.0));
                entities.push(entity);
            }
            for entity in &entities {
                world.add_component(*entity, Velocity(1.0, 0.0, 0.0));
            }
            for entity in &entities {
                world.remove_component::<Velocity>(*entity);
            }
            black_box(world.entity_count());
        })
    });
}

criterion_group!(benches, bench_spawn_100k, bench_query_100k, bench_despawn_10k, bench_add_remove_component);
criterion_main!(benches);
