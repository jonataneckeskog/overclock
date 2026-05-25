mod api;
mod server;

use api::{CarCommand, CarSensorData, create_bevy_channels};
use bevy::prelude::*;
use tokio::runtime::Runtime;

#[derive(Component)]
struct Movable {
    velocity: f32,
    acceleration: f32,
}

#[derive(Component)]
struct Steerable {
    steering_angle: f32,
}

#[derive(Component)]
struct Car;

#[derive(Component)]
struct Obstacle;

#[derive(Resource)]
struct ControlChannels {
    command_rx: tokio::sync::mpsc::Receiver<CarCommand>,
    sensor_tx: tokio::sync::mpsc::Sender<CarSensorData>,
}

fn main() {
    let (bevy_channels, command_tx, sensor_rx) = create_bevy_channels();

    // Spawn the WebSocket server in a background Tokio runtime
    let rt = Runtime::new().unwrap();
    rt.spawn(async move {
        println!("Starting Car API Server on ws://127.0.0.1:9001");
        if let Err(e) = server::run_server("127.0.0.1:9001", command_tx, sensor_rx).await {
            eprintln!("Server error: {}", e);
        }
    });

    App::new()
        .add_plugins(DefaultPlugins)
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
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(40.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Car,
        Movable {
            velocity: 0.0,
            acceleration: 0.0,
        },
        Steerable {
            steering_angle: 0.0,
        },
    ));

    for i in 0..15 {
        let x = (i as f32 * 400.0) % 2000.0 - 1000.0;
        let y = (i as f32 * 250.0) % 1000.0 - 500.0;
        commands.spawn((
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.0),
            Obstacle,
        ));
    }
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

fn broadcast_sensors(
    channels: Res<ControlChannels>,
    car_query: Query<(&Movable, &Transform), With<Car>>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
) {
    if let Some(car) = car_query.iter().next() {
        let (movable, car_transform) = car;
        let car_pos = car_transform.translation.truncate();

        let mut min_dist = 1000.0;
        for obs_transform in obstacle_query.iter() {
            let dist = car_pos.distance(obs_transform.translation.truncate());
            if dist < min_dist {
                min_dist = dist;
            }
        }

        let data = CarSensorData {
            velocity: movable.velocity,
            proximity: min_dist,
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
