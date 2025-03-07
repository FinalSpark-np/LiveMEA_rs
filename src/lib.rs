use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};
use url::Url;

/// The WebSocket URL of the MEA server.
const MEA_SERVER_URL: &str = "wss://livemeaservice2.alpvision.com";

/// Represents live data recorded from MEA devices.
#[derive(Serialize, Deserialize, Debug)]
pub struct LiveData {
    timestamp: String,
    data: Vec<Vec<f32>>,
}

/// A struct to handle live MEA data.
pub struct LiveMEA {
    mea_id: u32,
}

impl LiveMEA {
    pub fn new(mea_id: u32) -> Self {
        Self { mea_id }
    }

    /// Connects to the WebSocket server and listens for live data.
    async fn listen_socket_events(&self) -> Result<LiveData, Box<dyn std::error::Error>> {
        let url = Url::parse(MEA_SERVER_URL)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();

        write.send(Message::Text(self.mea_id.to_string())).await?;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Binary(buffer)) => {
                    let data: Vec<f32> = buffer
                        .chunks(4)
                        .map(|chunk| f32::from_ne_bytes(chunk.try_into().unwrap()))
                        .collect();
                    let elec_data = data.chunks(4096).map(|chunk| chunk.to_vec()).collect();
                    return Ok(LiveData {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        data: elec_data,
                    });
                }
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                    break;
                }
                _ => {}
            }
        }

        Err("Failed to receive live data".into())
    }

    /// Records a single sample of live data.
    pub async fn record_sample(&self) -> Result<LiveData, Box<dyn std::error::Error>> {
        self.listen_socket_events().await
    }

    /// Records multiple samples of live data.
    pub async fn record_n_samples(&self, n: usize) -> Result<Vec<LiveData>, Box<dyn std::error::Error>> {
        let mut data = Vec::with_capacity(n);
        for _ in 0..n {
            let sample = self.listen_socket_events().await?;
            data.push(sample);
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_record_sample() {
        let rt = Runtime::new().unwrap();
        let live_mea = LiveMEA::new(1);

        rt.block_on(async {
            match live_mea.record_sample().await {
                Ok(data) => println!("Single sample: {:?}", data),
                Err(e) => eprintln!("Error recording single sample: {:?}", e),
            }
        });
    }

    #[test]
    fn test_record_n_samples() {
        let rt = Runtime::new().unwrap();
        let live_mea = LiveMEA::new(1);

        rt.block_on(async {
            match live_mea.record_n_samples(5).await {
                Ok(data) => println!("Multiple samples: {:?}", data),
                Err(e) => eprintln!("Error recording multiple samples: {:?}", e),
            }
        });
    }
}
