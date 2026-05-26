mod api;
mod server;
mod world;

use api::{CarCommand, CarSensorData, create_bevy_channels};
use bevy::prelude::*;
use tokio::runtime::Runtime;
use world::{Hitbox, SpawnSettings, despawn_system, move_npc_cars, spawning_system};

// Generic movement data
#[derive(Component)]
pub struct Movable {
    pub velocity: f32,
    pub acceleration: f32,
}

// Generic steering data
#[derive(Component)]
pub struct Steerable {
    pub steering_angle: f32,
}

// A simple marker component to identify cars specifically
#[derive(Component)]
pub struct Car;

#[derive(Resource)]
struct ControlChannels {
    command_rx: tokio::sync::mpsc::Receiver<CarCommand>,
    sensor_tx: tokio::sync::mpsc::Sender<CarSensorData>,
}

fn main() {
    let (bevy_channels, command_tx, sensor_rx) = create_bevy_channels();

    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        println!("Starting Car API Server on ws://127.0.0.1:9001");
        if let Err(e) = server::run_server("127.0.0.1:9001", command_tx, sensor_rx).await {
            eprintln!("Server error: {}", e);
        }
    });

    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SpawnSettings>()
        .insert_resource(ControlChannels {
            command_rx: bevy_channels.command_rx,
            sensor_tx: bevy_channels.sensor_tx,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                read_commands,
                apply_physics,
                broadcast_sensors,
                camera_follow,
                spawning_system,
                despawn_system,
                move_npc_cars,
                collision_system,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Player Car
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(40.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Car,
        Movable {
            velocity: 10.0,
            acceleration: 0.0,
        },
        Steerable {
            steering_angle: 0.0,
        },
        Hitbox {
            size: Vec2::new(35.0, 15.0),
        },
    ));
}

fn read_commands(
    mut channels: ResMut<ControlChannels>,
    mut query: Query<(&mut Movable, &mut Steerable), With<Car>>,
) {
    while let Ok(command) = channels.command_rx.try_recv() {
        if let Some(car) = query.iter_mut().next() {
            let (mut movable, mut steerable) = car;
            match command {
                CarCommand::SetThrottle(t) => movable.acceleration = t.clamp(-1.0, 1.0),
                CarCommand::SetSteering(s) => steerable.steering_angle = s.clamp(-1.0, 1.0),
            }
        }
    }
}

fn apply_physics(time: Res<Time>, mut query: Query<(&mut Movable, &Steerable, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut movable, steerable, mut transform) in query.iter_mut() {
        movable.velocity *= 0.99;
        movable.velocity += movable.acceleration * 250.0 * dt;
        let turn_radius =
            steerable.steering_angle * movable.velocity.abs().min(100.0) / 100.0 * 3.5;
        transform.rotate_z(turn_radius * dt);
        let forward = transform.right();
        transform.translation += forward * movable.velocity * dt;
    }
}

fn collision_system(
    mut car_query: Query<(&mut Movable, &Transform, &Hitbox), With<Car>>,
    other_query: Query<(&Transform, &Hitbox), Without<Car>>,
) {
    if let Some((mut movable, car_transform, car_hitbox)) = car_query.iter_mut().next() {
        let car_pos = car_transform.translation.truncate();
        let car_half_size = car_hitbox.size / 2.0;

        for (other_transform, other_hitbox) in other_query.iter() {
            let other_pos = other_transform.translation.truncate();
            let other_half_size = other_hitbox.size / 2.0;

            let delta = (car_pos - other_pos).abs();
            let overlap = (car_half_size + other_half_size) - delta;

            if overlap.x > 0.0 && overlap.y > 0.0 {
                movable.velocity = -movable.velocity * 0.5;
            }
        }
    }
}

fn broadcast_sensors(
    channels: Res<ControlChannels>,
    car_query: Query<(&Movable, &Transform), With<Car>>,
    obstacle_query: Query<&Transform, (With<Hitbox>, Without<Car>)>,
) {
    if let Some(car) = car_query.iter().next() {
        let (movable, car_transform) = car;
        let car_pos = car_transform.translation.truncate();
        let car_rot = car_transform.rotation.to_euler(EulerRot::XYZ).2;

        // 4 directions: 0: Right, 1: Up (Forward), 2: Left, 3: Down (Backward)
        // Relative to car rotation. Forward is transform.right() (+X local)
        let directions = [
            car_rot - std::f32::consts::FRAC_PI_2, // Right
            car_rot,                               // Up (Forward)
            car_rot + std::f32::consts::FRAC_PI_2, // Left
            car_rot + std::f32::consts::PI,        // Down (Backward)
        ];

        let mut proximity_sensors = vec![1000.0; 4];

        for obs_transform in obstacle_query.iter() {
            let obs_pos = obs_transform.translation.truncate();
            let to_obs = obs_pos - car_pos;
            let dist = to_obs.length();

            if dist < 1000.0 {
                let angle_to_obs = to_obs.to_angle();

                for (i, &dir_angle) in directions.iter().enumerate() {
                    let mut diff = (angle_to_obs - dir_angle).abs();
                    while diff > std::f32::consts::PI {
                        diff = (diff - std::f32::consts::TAU).abs();
                    }

                    // If obstacle is within 45 degrees of the sensor direction
                    if diff < std::f32::consts::FRAC_PI_4 {
                        if dist < proximity_sensors[i] {
                            proximity_sensors[i] = dist;
                        }
                    }
                }
            }
        }

        let data = CarSensorData {
            velocity: movable.velocity,
            rotation: car_rot,
            proximity_sensors,
        };
        let _ = channels.sensor_tx.try_send(data);
    }
}

fn camera_follow(
    car_query: Query<&Transform, (With<Car>, Without<Camera>)>,
    mut cam_query: Query<&mut Transform, With<Camera>>,
) {
    if let Some(car_transform) = car_query.iter().next() {
        if let Some(mut cam_transform) = cam_query.iter_mut().next() {
            cam_transform.translation.x = car_transform.translation.x;
            cam_transform.translation.y = car_transform.translation.y;
        }
    }
}
