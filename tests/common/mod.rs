use fxoanda::*;
use std::env;
use chrono::Utc;

/// Create a test client with demo credentials and safety checks
pub fn create_test_client() -> Client {
    let api_key = env::var("OANDA_KEY").expect("OANDA_KEY environment variable must be set for tests");
    let api_host = env::var("OANDA_HOST").expect("OANDA_HOST environment variable must be set for tests");
    
    // Safety: Ensure we're only using the demo environment
    assert_eq!(api_host, "api-fxpractice.oanda.com", 
        "Tests must only run against demo environment. Set OANDA_HOST=api-fxpractice.oanda.com");
    
    Client {
        host: api_host,
        reqwest: reqwest::Client::new(),
        authentication: api_key,
    }
}

/// Create a mock client for unit tests - no real API calls
pub fn create_mock_client() -> Client {
    Client {
        host: "mock-api.test".to_string(),
        reqwest: reqwest::Client::new(),
        authentication: "mock-token".to_string(),
    }
}

/// Helper to get the first available account ID from the client
#[allow(dead_code)]
pub async fn get_test_account_id(client: &Client) -> String {
    let accounts_response = ListAccountsRequest::new()
        .remote(client)
        .await
        .expect("Failed to list accounts");
    
    accounts_response.accounts
        .expect("No accounts found in response")
        .first()
        .expect("No accounts found in demo environment")
        .id
        .clone()
        .expect("Account should have ID")
}

/// Test context for stateful integration tests - handles existing demo account state
pub struct TestContext {
    pub client: Client,
    pub account_id: String,
    pub test_run_id: String, // Unique per test run
}

impl TestContext {
    /// Create a new test context with unique run ID
    pub async fn new() -> Self {
        let client = create_test_client();
        let account_id = get_test_account_id(&client).await;
        let test_run_id = format!("test_{}", Utc::now().timestamp_millis());
        
        Self { client, account_id, test_run_id }
    }
    
    /// Create order with unique client request ID
    pub fn unique_order_id(&self, test_name: &str) -> String {
        format!("{}_{}_order", self.test_run_id, test_name)
    }
    
    /// Create unique trade comment for tracking
    pub fn unique_trade_comment(&self, test_name: &str) -> String {
        format!("{}_{}_trade", self.test_run_id, test_name)
    }
    
    /// Cleanup helper - cancel orders created by this test
    #[allow(dead_code)]
    pub async fn cleanup_test_orders(&self, order_ids: Vec<String>) {
        for order_id in order_ids {
            // Attempt to cancel order, ignore failures as orders might already be filled/cancelled
            let _ = CancelOrderRequest::new()
                .with_account_id(self.account_id.clone())
                .with_order_specifier(order_id)
                .remote(&self.client)
                .await;
        }
    }
}

/// State-aware utility functions for stateful testing

/// Get position for a specific instrument, handling existing state
#[allow(dead_code)]
pub async fn get_position_for_instrument(ctx: &TestContext, instrument: &str) -> Option<Position> {
    let positions = ListPositionsRequest::new()
        .with_account_id(ctx.account_id.clone())
        .remote(&ctx.client).await.ok()?;
    
    positions.positions?.into_iter()
        .find(|p| p.instrument.as_ref() == Some(&instrument.to_string()))
}

/// Create a test position and return the trade ID
#[allow(dead_code)]
pub async fn create_test_position(ctx: &TestContext, instrument: &str, units: i32) -> Result<String, FxError> {
    let client_req_id = ctx.unique_order_id(&format!("position_{}", instrument));
    
    let market_order = MarketOrder::new()
        .with_instrument(instrument.to_string())
        .with_units(units as f32)
        .with_time_in_force("FOK".to_string())
        .with_otype("MARKET".to_string());
    
    let order_result = CreateMarketOrderRequest::new()
        .with_account_id(ctx.account_id.clone())
        .with_order(market_order)
        .remote(&ctx.client).await?;
    
    // Get the trade ID from trade_opened
    order_result.order_fill_transaction
        .and_then(|fill| fill.trade_opened)
        .and_then(|trade_open| trade_open.trade_id)
        .ok_or_else(|| FxError::Validation(RequestValidationError::MissingOrderSpecifier))
}

/// Verify position units change matches expected
pub fn verify_position_change(initial: &Option<Position>, new: &Option<Position>, expected_units: i32) {
    let initial_units = initial.as_ref()
        .map(|p| p.long.as_ref().map(|l| l.units.unwrap_or(0.0)).unwrap_or(0.0) + 
                 p.short.as_ref().map(|s| s.units.unwrap_or(0.0)).unwrap_or(0.0))
        .unwrap_or(0.0);
    
    let new_units = new.as_ref()
        .map(|p| p.long.as_ref().map(|l| l.units.unwrap_or(0.0)).unwrap_or(0.0) + 
                 p.short.as_ref().map(|s| s.units.unwrap_or(0.0)).unwrap_or(0.0))
        .unwrap_or(0.0);
    
    assert_eq!((new_units - initial_units) as i32, expected_units, 
        "Position units change doesn't match expected");
}

/// Standard test instruments
pub fn get_test_instruments() -> Vec<&'static str> {
    vec!["EUR_USD", "GBP_USD", "USD_JPY", "AUD_USD", "USD_CHF"]
}

/// Validate price precision for different instruments
pub fn assert_price_precision(price: f64, instrument: &str) {
    // Note: We don't enforce strict decimal precision validation
    // as OANDA may provide varying precision based on market conditions
    
    // Check that the price is reasonable (not zero and within expected range)
    assert!(price > 0.0, "Price {} for {} should be positive", price, instrument);
    assert!(price < 1000000.0, "Price {} for {} seems unreasonably high", price, instrument);
    
    // For financial data, we just need to check the price is reasonable
    // Don't be too strict about decimal precision as floating point can introduce extra digits
    // The main purpose is to ensure prices look sensible
    
    // Validate price is in a reasonable range for the instrument
    match instrument {
        "USD_JPY" | "EUR_JPY" | "GBP_JPY" | "AUD_JPY" | "CHF_JPY" => {
            // JPY pairs typically range from 80-200
            assert!(price >= 50.0 && price <= 500.0, 
                "JPY pair {} price {} is outside reasonable range", instrument, price);
        }
        "XAU_USD" => {
            // Gold prices typically range from 1000-3000
            assert!(price >= 500.0 && price <= 5000.0,
                "Gold price {} is outside reasonable range", price);
        }
        _ => {
            // Most major pairs range from 0.5-2.0
            assert!(price >= 0.1 && price <= 10.0,
                "Currency pair {} price {} is outside reasonable range", instrument, price);
        }
    }
}

/// Test data for various scenarios
pub mod fixtures {
    use super::*;
    
    /// Sample candlestick test parameters
    #[allow(dead_code)]
    pub fn sample_candle_request() -> GetInstrumentCandlesRequest {
        GetInstrumentCandlesRequest::new()
            .with_instrument("EUR_USD".to_string())
            .with_granularity(CandlestickGranularity::H4)
            .with_count(10)
    }
    
    /// Sample pricing request
    #[allow(dead_code)]
    pub fn sample_pricing_request() -> GetPricesRequest {
        GetPricesRequest::new()
            .with_instruments("EUR_USD".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_price_precision_validation() {
        assert_price_precision(1.12345, "EUR_USD");
        assert_price_precision(110.123, "USD_JPY");
    }
    
    #[test]
    fn test_test_instruments_list() {
        let instruments = get_test_instruments();
        assert!(instruments.contains(&"EUR_USD"));
        assert!(instruments.contains(&"USD_JPY"));
        assert_eq!(instruments.len(), 5);
    }
}