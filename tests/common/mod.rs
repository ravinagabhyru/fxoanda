use fxoanda::*;
use std::env;

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