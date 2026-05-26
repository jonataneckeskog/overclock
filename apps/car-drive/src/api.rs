use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CarCommand {
    SetThrottle(f32),
    SetSteering(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarSensorData {
    pub velocity: f32,
    pub rotation: f32,
    pub proximity_sensors: Vec<f32>,
}

/// The internal Bevy-side channel ends.
pub struct BevyChannels {
    pub command_rx: mpsc::Receiver<CarCommand>,
    pub sensor_tx: mpsc::Sender<CarSensorData>,
}

/// Creates the communication channels for the Bevy simulation.
pub fn create_bevy_channels() -> (
    BevyChannels,
    mpsc::Sender<CarCommand>,
    mpsc::Receiver<CarSensorData>,
) {
    let (command_tx, command_rx) = mpsc::channel(100);
    let (sensor_tx, sensor_rx) = mpsc::channel(100);

    (
        BevyChannels {
            command_rx,
            sensor_tx,
        },
        command_tx,
        sensor_rx,
    )
}
