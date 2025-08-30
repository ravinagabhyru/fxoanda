//! This is an unofficial [Oanda](https://wwww.oanda.com/) API client. This client is still
//! an experimental work in progress however it is reasonably functional.
//!
//! The client implements the Oanda V20 REST API with structured error handling and type-safe requests.
//! For the latest API documentation, refer to the [official OANDA v20 API documentation](https://developer.oanda.com/rest-live-v20/introduction/).
//! The current state of the client API is low-level but usable however I would like to see a more 
//! ergonomic layer developed on top.
//!
//! # Installation
//!
//! ```bash
//! $ cargo add fxoanda
//! ```
//!
//! # Example: Get Candlestick Data
//!
//! This example shows how to create a client and request candlestick data for the `EUR_USD` pair.
//! It requires the `OANDA_KEY` and `OANDA_HOST` environment variables to be set.
//!
//! ```no_run
//! # use std::env;
//! # use fxoanda::*;
//! #
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let api_key = env::var("OANDA_KEY")
//!         .expect("OANDA_KEY environment variable must be set");
//!     let api_host = env::var("OANDA_HOST")
//!         .expect("OANDA_HOST environment variable must be set");
//!
//!     // Create a new client
//!     let client = fxoanda::Client {
//!         host: api_host,
//!         reqwest: reqwest::Client::new(),
//!         authentication: api_key,
//!     };
//!
//!     // Build a request for H4 candles for EUR_USD
//!     let request = fxoanda::GetInstrumentCandlesRequest::new()
//!         .with_instrument("EUR_USD".to_string())
//!         .with_granularity(CandlestickGranularity::H4)
//!         .with_count(10);
//!
//!     // Execute the request
//!     match request.remote(&client).await {
//!         Ok(response) => {
//!             println!("Successfully received candlestick data.");
//!             if let Some(candles) = response.candles {
//!                 println!("Number of candles: {}", candles.len());
//!                 // Process your candle data here
//!             }
//!         },
//!         Err(e) => {
//!             eprintln!("Error fetching candles: {:#?}", e);
//!         }
//!     };
//!
//!     Ok(())
//! }
//! ```
//!
//! # Warning
//!
//! Forex markets are extremely risky. Automated trading is also extremely risky.
//! This project is extremely risky. Market conditions, news events, or software bugs
//! can wipe out your account in an instant.
//!
//! # Disclaimer
//!
//! Use this project at your own risk. The maintainers of this project make no
//! claims as to this product being fit for purpose. In fact, the maintainers of this
//! project are telling you that you shouldn't use this project.
//!

#![crate_type = "lib"]
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
//#[macro_use]
extern crate serde;
//#[macro_use]
extern crate chrono;
extern crate fxoanda_definitions;
extern crate fxoanda_serdes;
extern crate serde_json;
extern crate time;

pub mod account;
pub mod client;
pub mod errors;
pub mod instrument;
pub use self::account::*;
pub use self::client::*;
pub use self::errors::{RequestValidationError, FxError};
pub use self::instrument::*;
pub use fxoanda_definitions::*;
pub use fxoanda_serdes::*;
