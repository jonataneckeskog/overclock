use crate::api::{CarCommand, CarSensorData};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::accept_async;

pub async fn run_server(
    addr: &str,
    command_tx: mpsc::Sender<CarCommand>,
    mut sensor_rx: mpsc::Receiver<CarSensorData>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    let (broadcast_tx, _) = broadcast::channel::<CarSensorData>(100);

    let b_tx = broadcast_tx.clone();
    tokio::spawn(async move {
        while let Some(sensor_data) = sensor_rx.recv().await {
            let _ = b_tx.send(sensor_data);
        }
    });

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let cmd_tx = command_tx.clone();
        let mut b_rx = broadcast_tx.subscribe();

        // Task to handle incoming WebSocket messages (Commands)
        let mut read_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                if msg.is_text() {
                    if let Ok(cmd) = serde_json::from_str::<CarCommand>(&msg.to_text().unwrap()) {
                        let _ = cmd_tx.send(cmd).await;
                    }
                }
            }
        });

        // Task to handle outgoing WebSocket messages (Sensors)
        let mut write_task = tokio::spawn(async move {
            while let Ok(sensor_data) = b_rx.recv().await {
                if let Ok(json) = serde_json::to_string(&sensor_data) {
                    if ws_sender
                        .send(tokio_tungstenite::tungstenite::Message::Text(json.into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        });

        // Wait for either task to finish
        tokio::spawn(async move {
            tokio::select! {
                _ = &mut read_task => write_task.abort(),
                _ = &mut write_task => read_task.abort(),
            }
        });
    }

    Ok(())
}
