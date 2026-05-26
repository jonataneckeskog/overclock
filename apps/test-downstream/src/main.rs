use downstream_rs::pipeline;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CarCommand {
    SetThrottle(f32),
    SetSteering(f32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CarSensorData {
    pub velocity: f32,
    pub rotation: f32,
    pub proximity_sensors: Vec<f32>,
}

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:9001";
    println!("Connecting to car-drive simulation at {}...", url);

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, read) = ws_stream.split();

    let (tx, mut rx) = mpsc::channel::<Vec<CarCommand>>(100);

    tokio::spawn(async move {
        while let Some(cmds) = rx.recv().await {
            for cmd in cmds {
                if let Ok(json) = serde_json::to_string(&cmd) {
                    if write.send(Message::Text(json.into())).await.is_err() {
                        return;
                    }
                }
            }
        }
    });

    let sensor_stream = read.filter_map(|msg| async {
        if let Ok(Message::Text(text)) = msg {
            serde_json::from_str::<CarSensorData>(&text).ok()
        } else {
            None
        }
    });

    println!("Connected! Driving car North...");

    pipeline![
        pipe(|sensor: CarSensorData| {
            let right = sensor.proximity_sensors[0];
            let forward = sensor.proximity_sensors[1];
            let left = sensor.proximity_sensors[2];

            let mut throttle = 0.8;
            let mut steering;

            // 1. North-seeking logic (Target angle: PI/2)
            let target_north = std::f32::consts::FRAC_PI_2;
            let mut angle_diff = target_north - sensor.rotation;

            // Normalize angle difference to [-PI, PI]
            while angle_diff > std::f32::consts::PI {
                angle_diff -= std::f32::consts::TAU;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += std::f32::consts::TAU;
            }

            // Gently steer towards North
            steering = (angle_diff * 0.5).clamp(-0.2, 0.2);

            // 2. Override with Collision Avoidance
            if forward < 300.0 {
                throttle = 0.3;
                if left > right {
                    steering = 0.8;
                } else {
                    steering = -0.8;
                }
            } else {
                if left < 150.0 {
                    steering = -0.4;
                } else if right < 150.0 {
                    steering = 0.4;
                }
            }

            Some(vec![
                CarCommand::SetThrottle(throttle),
                CarCommand::SetSteering(steering),
            ])
        }),
        sink(move |cmds| {
            let _ = tx.try_send(cmds);
        })
    ]
    .run(sensor_stream)
    .await
    .unwrap();
}
