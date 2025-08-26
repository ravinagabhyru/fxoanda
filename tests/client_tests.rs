mod common;

use fxoanda::*;
use std::env;
use common::*;

#[tokio::test]
async fn test_client_initialization_valid_credentials() {
    let client = create_test_client();
    
    assert!(!client.host.is_empty());
    assert!(!client.authentication.is_empty());
    assert_eq!(client.host, "api-fxpractice.oanda.com");
}

#[test]
fn test_client_initialization_invalid_host() {
    // Test direct client creation with invalid host (bypassing create_test_client)
    let api_key = env::var("OANDA_KEY").expect("OANDA_KEY environment variable must be set");
    
    let client = Client {
        host: "api-fxtrade.oanda.com".to_string(),
        reqwest: reqwest::Client::new(),
        authentication: api_key,
    };
    
    // This should work fine - the client accepts any host, only create_test_client enforces demo
    assert_eq!(client.host, "api-fxtrade.oanda.com");
    assert!(!client.authentication.is_empty());
}

#[tokio::test]
async fn test_demo_host_enforcement() {
    let api_host = env::var("OANDA_HOST").expect("OANDA_HOST must be set");
    assert_eq!(api_host, "api-fxpractice.oanda.com");
}

#[tokio::test]
async fn test_authentication_header_construction() {
    let client = create_test_client();
    
    // Test that authentication header is properly constructed
    // We can't easily test the actual header construction without making a request,
    // but we can verify the authentication field is set
    assert!(!client.authentication.is_empty());
    assert!(client.authentication.len() > 10); // Basic sanity check for token length
}

#[tokio::test]
async fn test_basic_connectivity() {
    let client = create_test_client();
    
    // Test basic connectivity with a simple accounts request
    let result = ListAccountsRequest::new()
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Basic connectivity test failed: {:?}", result);
    
    let accounts = result.unwrap();
    assert!(!accounts.accounts.expect("Should have accounts").is_empty(), "Should have at least one demo account");
}

#[tokio::test]
async fn test_client_timeout_handling() {
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
async fn test_client_methods_exist() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test that the client has the expected convenience methods
    let accounts_result = client.accounts(ListAccountsRequest::new()).await;
    assert!(accounts_result.is_ok());
    
    let pricing_result = client.pricing(
        GetPricesRequest::new()
            .with_account_id(account_id)
            .with_instruments("EUR_USD".to_string())
    ).await;
    assert!(pricing_result.is_ok());
}

#[tokio::test]
async fn test_multiple_concurrent_requests() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test concurrent requests don't interfere with each other
    let accounts_future = ListAccountsRequest::new().remote(&client);
    let pricing_future = GetPricesRequest::new()
        .with_account_id(account_id)
        .with_instruments("EUR_USD".to_string())
        .remote(&client);
    let candles_future = GetInstrumentCandlesRequest::new()
        .with_instrument("EUR_USD".to_string())
        .with_granularity(CandlestickGranularity::H1)
        .with_count(5)
        .remote(&client);
    
    let (accounts_result, pricing_result, candles_result) = futures::future::join3(
        accounts_future,
        pricing_future, 
        candles_future
    ).await;
    
    // All requests should succeed
    assert!(accounts_result.is_ok(), "Accounts request failed: {:?}", accounts_result);
    assert!(pricing_result.is_ok(), "Pricing request failed: {:?}", pricing_result);
    assert!(candles_result.is_ok(), "Candles request failed: {:?}", candles_result);
}