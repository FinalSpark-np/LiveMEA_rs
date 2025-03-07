# FinalSpark-RS

FinalSpark-RS is a Rust library for live data recording from MEA (Microelectrode Array) devices. It enables real-time data acquisition and processing from MEA devices over a network connection.

## Features
- Connects to an MEA server to retrieve live data.
- Supports both single-sample and multi-sample data recording.
- Uses `tokio` for asynchronous networking.
- Parses and structures incoming data into a `LiveData` format.

## Installation

Add `finalspark-rs` to your `Cargo.toml`:

```toml
[dependencies]
finalspark-rs = { git = "https://github.com/maidenlabs/finalspark-rs.git" }
```

Or clone the repository and use it as a local dependency.

## Usage

```rust
use finalspark_rs::LiveMEA;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let live_mea = LiveMEA::new(1);

    rt.block_on(async {
        match live_mea.record_sample().await {
            Ok(data) => println!("Single sample: {:?}", data),
            Err(e) => eprintln!("Error recording single sample: {:?}", e),
        }
    });
}
```

### Recording Multiple Samples
```rust
use finalspark_rs::LiveMEA;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let live_mea = LiveMEA::new(1);

    rt.block_on(async {
        match live_mea.record_n_samples(5).await {
            Ok(data) => println!("Multiple samples: {:?}", data),
            Err(e) => eprintln!("Error recording multiple samples: {:?}", e),
        }
    });
}
```

## Dependencies
This library relies on the following Rust crates:
- `tokio` (for async networking)
- `socket2` (for low-level socket handling)
- `serde` & `serde_json` (for data serialization)
- `chrono` (for timestamping data)
- `tokio-tungstenite` (for WebSocket communication)

## License
This project is licensed under the MIT License. See `LICENSE` for details.

## Contributing
Contributions are welcome! Please submit an issue or pull request on [GitHub](https://github.com/maidenlabs/finalspark-rs).

