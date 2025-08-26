mod common;

use fxoanda::*;
use common::*;

#[tokio::test]
async fn test_invalid_authentication() {
    let mut client = create_test_client();
    client.authentication = "invalid_token".to_string();
    
    let result = ListAccountsRequest::new()
        .remote(&client)
        .await;
    
    // OANDA API returns success but with empty accounts for invalid authentication
    if result.is_ok() {
        let accounts = result.unwrap();
        // Should have no accounts or None accounts field with invalid auth
        assert!(accounts.accounts.is_none() || accounts.accounts.as_ref().unwrap().is_empty(),
            "Invalid authentication should return no accounts");
    }
    // If it fails, that's also acceptable behavior for invalid auth
}

#[tokio::test]
async fn test_invalid_account_id() {
    let client = create_test_client();
    
    let result = GetAccountRequest::new()
        .with_account_id("invalid_account_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API returns success but with empty account data for invalid account IDs
    assert!(result.is_ok(), "Request should succeed even with invalid account ID");
    let response = result.unwrap();
    assert!(response.account.is_none(), "Account should be None for invalid account ID");
}

#[tokio::test]
async fn test_invalid_instrument() {
    let client = create_test_client();
    
    let result = GetInstrumentCandlesRequest::new()
        .with_instrument("INVALID_PAIR".to_string())
        .with_granularity(CandlestickGranularity::H1)
        .with_count(5)
        .remote(&client)
        .await;
    
    // OANDA API may return success with empty candles or an error for invalid instruments
    if result.is_ok() {
        let response = result.unwrap();
        // Should have no candles or None candles field for invalid instrument
        assert!(response.candles.is_none() || response.candles.as_ref().unwrap().is_empty(),
            "Invalid instrument should return no candles");
    }
    // If it fails, that's also acceptable behavior for invalid instruments
}

#[tokio::test]
async fn test_network_timeout_handling() {
    let mut client = create_test_client();
    
    // Create a client with very short timeout
    client.reqwest = reqwest::ClientBuilder::new()
        .timeout(std::time::Duration::from_millis(1))
        .build()
        .unwrap();
    
    // This should timeout
    let result = ListAccountsRequest::new()
        .remote(&client)
        .await;
    
    assert!(result.is_err(), "Request should have timed out");
}

#[tokio::test]
async fn test_malformed_host() {
    let mut client = create_test_client();
    client.host = "nonexistent.domain.invalid".to_string();
    
    let result = ListAccountsRequest::new()
        .remote(&client)
        .await;
    
    assert!(result.is_err(), "Request to invalid host should fail");
}