//! A Rust library for recording live data from MEA (Microelectrode Array) devices.
//!
//! This crate provides functionality to connect to and record data from MEA devices
//! through a WebSocket connection. It supports both single and multiple sample recordings.
//!
//! # Example
//!
//! ```rust
//! use finalspark_rs::LiveMEA;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mea = LiveMEA::new();
//!     
//!     // Record a single sample from MEA device 1
//!     let sample = mea.record_sample(1).await?;
//!     println!("Recorded {} electrodes", sample.data.len());
//!     
//!     // Record multiple samples
//!     let samples = mea.record_n_samples(1, 3).await?;
//!     println!("Recorded {} samples", samples.len());
//!     
//!     Ok(())
//! }
//! ```

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

/// The Socket.IO URL of the MEA server.
const MEA_SERVER_URL: &str =
    "wss://livemeaservice2.alpvision.com/socket.io/?EIO=4&transport=websocket";

/// Internal struct for parsing Socket.IO handshake messages.
/// This is used to validate the initial connection response.
#[derive(Deserialize)]
struct SocketIOHandshake {}

/// Represents live data recorded from MEA devices.
///
/// Each `LiveData` instance contains:
/// * A timestamp string in RFC3339 format
/// * A 2D array of electrode data where:
///   * The outer vector contains 32 electrodes
///   * Each inner vector contains 4096 samples per electrode
///
/// # Example
///
/// ```rust
/// use finalspark_rs::LiveData;
///
/// fn process_data(data: LiveData) {
///     println!("Timestamp: {}", data.timestamp);
///     println!("Number of electrodes: {}", data.data.len());
///     println!("Samples per electrode: {}", data.data[0].len());
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LiveData {
    /// The timestamp of when the data was recorded (RFC3339 format)
    pub timestamp: String,
    /// The electrode data as a 2D array [32][4096]
    pub data: Vec<Vec<f32>>,
}

/// Main struct for handling MEA device connections and data recording.
///
/// This struct provides methods to:
/// * Record single samples from MEA devices
/// * Record multiple samples in sequence
/// * Validate MEA device IDs
///
/// # Examples
///
/// ```rust
/// use finalspark_rs::LiveMEA;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mea = LiveMEA::new();
///     
///     // Record from MEA device 1
///     let data = mea.record_sample(1).await?;
///     
///     Ok(())
/// }
/// ```
pub struct LiveMEA {}

impl LiveMEA {
    /// Creates a new instance of the MEA handler.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use finalspark_rs::LiveMEA;
    ///
    /// let mea = LiveMEA::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }

    /// Validates that the MEA ID is within the acceptable range (1-4).
    ///
    /// # Arguments
    ///
    /// * `mea_id` - The ID of the MEA device to validate
    ///
    /// # Errors
    ///
    /// Returns an error if the MEA ID is not between 1 and 4.
    fn validate_mea_id(mea_id: u32) -> Result<(), Box<dyn std::error::Error>> {
        if mea_id < 1 || mea_id > 4 {
            return Err("MEA ID must be an integer in the range 1-4".into());
        }
        Ok(())
    }

    /// Records a single sample of live data from the specified MEA device.
    ///
    /// This method:
    /// 1. Connects to the MEA server
    /// 2. Performs Socket.IO handshake
    /// 3. Requests data from the specified device
    /// 4. Processes and returns the binary data
    ///
    /// # Arguments
    ///
    /// * `mea_id` - The ID of the MEA device to record from (1-4)
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either:
    /// * `LiveData` with the recorded sample
    /// * An error if the recording failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use finalspark_rs::LiveMEA;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mea = LiveMEA::new();
    ///     let data = mea.record_sample(1).await?;
    ///     println!("Recorded at: {}", data.timestamp);
    ///     Ok(())
    /// }
    /// ```
    pub async fn record_sample(&self, mea_id: u32) -> Result<LiveData, Box<dyn std::error::Error>> {
        Self::validate_mea_id(mea_id)?;
        let mea_index = mea_id - 1;

        let url = Url::parse(MEA_SERVER_URL)?;
        let (mut ws_stream, _) = connect_async(url).await?;

        // Handle Socket.IO handshake
        if let Some(msg) = ws_stream.next().await {
            match msg? {
                Message::Text(text) => {
                    if text.starts_with("0") {
                        let _handshake: SocketIOHandshake = serde_json::from_str(&text[1..])?;
                        ws_stream.send(Message::Text("40".to_string())).await?;
                        let mea_msg = format!("42[\"meaid\",{}]", mea_index);
                        ws_stream.send(Message::Text(mea_msg)).await?;
                    }
                }
                _ => return Err("Invalid handshake response".into()),
            }
        }

        while let Some(msg) = ws_stream.next().await {
            match msg? {
                Message::Text(text) => {
                    if text.starts_with("2") {
                        ws_stream.send(Message::Text("3".to_string())).await?;
                    }
                }
                Message::Binary(buffer) => {
                    let raw = buffer
                        .chunks(4)
                        .map(|chunk| f32::from_ne_bytes(chunk.try_into().unwrap()))
                        .collect::<Vec<f32>>();

                    // Check if we have data for one MEA (32 electrodes) or all MEAs (128 electrodes)
                    let start_idx = if raw.len() == 32 * 4096 {
                        0 // Single MEA data
                    } else if raw.len() == 128 * 4096 {
                        (mea_index as usize) * 32 * 4096 // Full dataset
                    } else {
                        return Err(format!(
                            "Unexpected data size: got {} values, expected {} or {}",
                            raw.len(),
                            32 * 4096,
                            128 * 4096
                        )
                        .into());
                    };

                    let elec_data: Vec<Vec<f32>> = (0..32)
                        .map(|i| raw[start_idx + i * 4096..start_idx + (i + 1) * 4096].to_vec())
                        .collect();

                    let live_data = LiveData {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        data: elec_data,
                    };

                    let _ = ws_stream.close(None).await;
                    return Ok(live_data);
                }
                Message::Close(_) => return Err("Server closed connection".into()),
                _ => continue,
            }
        }

        Err("Connection closed without receiving data".into())
    }

    /// Records multiple samples of live data from the specified MEA device.
    ///
    /// # Arguments
    ///
    /// * `mea_id` - The ID of the MEA device to record from (1-4)
    /// * `n` - The number of samples to record
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either:
    /// * A vector of `LiveData` instances
    /// * An error if the recording failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use finalspark_rs::LiveMEA;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mea = LiveMEA::new();
    ///     let samples = mea.record_n_samples(1, 3).await?;
    ///     println!("Recorded {} samples", samples.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn record_n_samples(
        &self,
        mea_id: u32,
        n: usize,
    ) -> Result<Vec<LiveData>, Box<dyn std::error::Error>> {
        Self::validate_mea_id(mea_id)?;
        let mut data = Vec::with_capacity(n);
        for _ in 0..n {
            let sample = self.record_sample(mea_id).await?;
            data.push(sample);
        }
        Ok(data)
    }
}

// Add test module
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_validate_mea_id() {
        assert!(LiveMEA::validate_mea_id(1).is_ok());
        assert!(LiveMEA::validate_mea_id(4).is_ok());
        assert!(LiveMEA::validate_mea_id(0).is_err());
        assert!(LiveMEA::validate_mea_id(5).is_err());
    }

    /// Helper function to validate electrode data structure
    fn validate_electrode_data(data: &LiveData) {
        assert_eq!(data.data.len(), 32, "Expected 32 electrodes");
        assert_eq!(
            data.data[0].len(),
            4096,
            "Expected 4096 samples per electrode"
        );

        // Validate data contains actual values
        let sum: f32 = data
            .data
            .iter()
            .flat_map(|electrode| electrode.iter())
            .sum();
        assert!(sum != 0.0, "Data appears to be all zeros");
    }

    #[tokio::test]
    async fn test_single_sample_recording() {
        let live_mea = LiveMEA::new();
        let start = Instant::now();

        let result = live_mea.record_sample(1).await;
        assert!(
            result.is_ok(),
            "Failed to record sample: {:?}",
            result.err()
        );

        let data = result.unwrap();
        println!("✓ Recorded sample in {:?}", start.elapsed());
        assert_eq!(data.data.len(), 32, "Expected 32 electrodes");
        assert_eq!(
            data.data[0].len(),
            4096,
            "Expected 4096 samples per electrode"
        );
    }

    #[tokio::test]
    async fn test_multiple_sample_recording() {
        let live_mea = LiveMEA::new();
        let start = Instant::now();

        let result = live_mea.record_n_samples(1, 3).await;
        assert!(
            result.is_ok(),
            "Failed to record samples: {:?}",
            result.err()
        );

        let samples = result.unwrap();
        println!(
            "✓ Recorded {} samples in {:?}",
            samples.len(),
            start.elapsed()
        );
        assert_eq!(samples.len(), 3, "Expected 3 samples");

        for sample in samples.iter() {
            assert_eq!(sample.data.len(), 32, "Expected 32 electrodes");
            assert_eq!(
                sample.data[0].len(),
                4096,
                "Expected 4096 samples per electrode"
            );
        }
    }

    #[tokio::test]
    async fn test_invalid_mea_id() {
        let live_mea = LiveMEA::new();
        let result = live_mea.record_sample(5).await;
        assert!(result.is_err(), "Expected error for invalid MEA ID");
    }

    #[tokio::test]
    async fn test_all_meas_single_sample() {
        let live_mea = LiveMEA::new();

        for mea_id in 1..=4 {
            let start = Instant::now();
            println!("\nTesting MEA {}", mea_id);

            match live_mea.record_sample(mea_id).await {
                Ok(data) => {
                    println!("✓ MEA {} recorded in {:?}", mea_id, start.elapsed());
                    validate_electrode_data(&data);
                }
                Err(e) => panic!("Failed to record from MEA {}: {:?}", mea_id, e),
            }
        }
    }

    #[tokio::test]
    async fn test_all_meas_multiple_samples() {
        let live_mea = LiveMEA::new();
        let samples_per_mea = 2;

        for mea_id in 1..=4 {
            let start = Instant::now();
            println!("\nTesting MEA {} with {} samples", mea_id, samples_per_mea);

            match live_mea.record_n_samples(mea_id, samples_per_mea).await {
                Ok(samples) => {
                    println!(
                        "✓ MEA {} recorded {} samples in {:?}",
                        mea_id,
                        samples.len(),
                        start.elapsed()
                    );

                    assert_eq!(
                        samples.len(),
                        samples_per_mea,
                        "Wrong number of samples from MEA {}",
                        mea_id
                    );

                    for (i, sample) in samples.iter().enumerate() {
                        println!("Validating MEA {} sample {}", mea_id, i + 1);
                        validate_electrode_data(sample);
                    }
                }
                Err(e) => panic!("Failed to record from MEA {}: {:?}", mea_id, e),
            }
        }
    }

    #[tokio::test]
    async fn test_sequential_recordings() {
        let live_mea = LiveMEA::new();
        let total_start = Instant::now();

        // Record one sample from each MEA in sequence
        for iteration in 1..=3 {
            println!("\nIteration {}", iteration);

            for mea_id in 1..=4 {
                let start = Instant::now();
                match live_mea.record_sample(mea_id).await {
                    Ok(data) => {
                        println!("✓ MEA {} recorded in {:?}", mea_id, start.elapsed());
                        validate_electrode_data(&data);
                    }
                    Err(e) => panic!(
                        "Failed to record from MEA {} on iteration {}: {:?}",
                        mea_id, iteration, e
                    ),
                }
            }
        }

        println!("\nAll recordings completed in {:?}", total_start.elapsed());
    }
}
