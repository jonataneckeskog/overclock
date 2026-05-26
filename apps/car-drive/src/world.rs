use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct Hitbox {
    pub size: Vec2,
}

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct NpcCar;

#[derive(Resource)]
pub struct SpawnSettings {
    pub spawn_timer: Timer,
    pub max_obstacles: usize,
    pub max_npcs: usize,
    pub spawn_radius_min: f32,
    pub spawn_radius_max: f32,
    pub despawn_radius: f32,
}

impl Default for SpawnSettings {
    fn default() -> Self {
        Self {
            spawn_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            max_obstacles: 80,
            max_npcs: 50,
            spawn_radius_min: 800.0,  // Just off screen
            spawn_radius_max: 1600.0, // A bit further
            despawn_radius: 1600.0,   // Clear once far away
        }
    }
}

pub fn spawning_system(
    mut commands: Commands,
    time: Res<Time>,
    mut settings: ResMut<SpawnSettings>,
    player_query: Query<&Transform, With<crate::Car>>,
    obstacle_query: Query<Entity, With<Obstacle>>,
    npc_query: Query<Entity, With<NpcCar>>,
) {
    settings.spawn_timer.tick(time.delta());
    if !settings.spawn_timer.just_finished() {
        return;
    }

    let player_transform = if let Some(t) = player_query.iter().next() {
        t
    } else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let mut rng = rand::rng();

    // 1. Spawn Obstacles in batches
    let current_obstacles = obstacle_query.iter().count();
    if current_obstacles < settings.max_obstacles {
        let spawn_count = (settings.max_obstacles - current_obstacles).min(10);
        for _ in 0..spawn_count {
            let spawn_pos = get_random_spawn_pos(
                player_pos,
                settings.spawn_radius_min,
                settings.spawn_radius_max,
                &mut rng,
            );
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.8, 0.2, 0.2),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_xyz(spawn_pos.x, spawn_pos.y, 0.0),
                Obstacle,
                Hitbox {
                    size: Vec2::new(30.0, 30.0),
                },
            ));
        }
    }

    // 2. Spawn NPC Cars in batches
    let current_npcs = npc_query.iter().count();
    if current_npcs < settings.max_npcs {
        let spawn_count = (settings.max_npcs - current_npcs).min(10);
        for _ in 0..spawn_count {
            let spawn_pos = get_random_spawn_pos(
                player_pos,
                settings.spawn_radius_min,
                settings.spawn_radius_max,
                &mut rng,
            );
            let rotation = rng.random_range(0.0..std::f32::consts::TAU);

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.8),
                    custom_size: Some(Vec2::new(40.0, 20.0)),
                    ..default()
                },
                Transform::from_xyz(spawn_pos.x, spawn_pos.y, 0.5)
                    .with_rotation(Quat::from_rotation_z(rotation)),
                NpcCar,
                crate::Movable {
                    velocity: rng.random_range(50.0..200.0),
                    acceleration: rng.random_range(0.1..0.5),
                },
                crate::Steerable {
                    steering_angle: rng.random_range(-1.0..1.0),
                },
                Hitbox {
                    size: Vec2::new(40.0, 20.0),
                },
            ));
        }
    }
}

pub fn despawn_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<crate::Car>>,
    entity_query: Query<(Entity, &Transform), Or<(With<Obstacle>, With<NpcCar>)>>,
    settings: Res<SpawnSettings>,
) {
    let player_transform = if let Some(t) = player_query.iter().next() {
        t
    } else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform) in entity_query.iter() {
        if transform.translation.truncate().distance(player_pos) > settings.despawn_radius {
            commands.entity(entity).despawn();
        }
    }
}

fn get_random_spawn_pos(
    center: Vec2,
    min_r: f32,
    max_r: f32,
    rng: &mut rand::prelude::ThreadRng,
) -> Vec2 {
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let dist = rng.random_range(min_r..max_r);
    center + Vec2::new(angle.cos() * dist, angle.sin() * dist)
}

pub fn move_npc_cars(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &crate::Movable), With<NpcCar>>,
) {
    let dt = time.delta_secs();
    for (mut transform, movable) in query.iter_mut() {
        let forward = transform.right();
        transform.translation += forward * movable.velocity * dt;
    }
}
