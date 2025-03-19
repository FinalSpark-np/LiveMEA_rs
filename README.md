# FinalSpark-RS

FinalSpark-RS is a Rust library for live data recording from MEA (Microelectrode Array) devices. It enables real-time data acquisition and processing from MEA devices over a network connection.

## Authors: Maiden Labs [[github](https://github.com/maidenlabs)]

Maiden Labs is a non profit user research lab committed to open-source scientific discovery through biocomputing. Utilizing decentralized technologies, we improve access to biocomputing research, advancing interdisciplinary AI and biocomputing related efforts for greater scientific impact and innovation.

## Features
- Connects to an MEA server to retrieve live data
- Supports both single-sample and multi-sample data recording
- Uses `tokio` for asynchronous networking
- Structured data output with timestamps and electrode readings

## Installation

Add `finalspark-rs` to your `Cargo.toml`:

```toml
[dependencies]
finalspark-rs = { git = "https://github.com/maidenlabs/finalspark-rs.git" }
tokio = { version = "1.0", features = ["full"] }
```

## Usage

### Basic Setup

```rust
use finalspark_rs::LiveMEA;

#[tokio::main]
async fn main() {
    let live_mea = LiveMEA::new();
    
    // Record a single sample
    match live_mea.record_sample(1).await {
        Ok(data) => {
            println!("Recorded sample with timestamp: {}", data.timestamp);
            println!("Number of electrodes: {}", data.data.len());
            println!("Samples per electrode: {}", data.data[0].len());
        },
        Err(e) => eprintln!("Error recording sample: {}", e),
    }
}
```

### Recording Multiple Samples

```rust
use finalspark_rs::LiveMEA;

#[tokio::main]
async fn main() {
    let live_mea = LiveMEA::new();

    match live_mea.record_n_samples(1, 5).await {
        Ok(samples) => {
            println!("Successfully recorded {} samples", samples.len());
            for (i, sample) in samples.iter().enumerate() {
                println!("Sample {} timestamp: {}", i + 1, sample.timestamp);
            }
        },
        Err(e) => eprintln!("Error recording samples: {}", e),
    }
}
```

## Data Structure

The `LiveData` struct contains:
- `timestamp`: String in RFC3339 format representing when the sample was recorded
- `data`: 2D vector containing electrode readings where:
  - First dimension: 32 electrodes
  - Second dimension: 4096 samples per electrode

## Dependencies
- `tokio` - Async runtime and networking
- `serde` & `serde_json` - Data serialization
- `chrono` - Timestamp handling
- `tokio-tungstenite` - WebSocket communication
- `url` - URL parsing and handling

## License
This project is licensed under the MIT License. See `LICENSE` for details.

## Contributing
Contributions are welcome! Please submit an issue or pull request on [GitHub](https://github.com/maidenlabs/finalspark-rs).
