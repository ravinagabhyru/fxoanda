mod common;

use fxoanda::*;
use common::*;
use serde_json;

#[tokio::test]
async fn test_json_serialization() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get some real data and test serialization
    let result = GetPricesRequest::new()
        .with_account_id(account_id)
        .with_instruments("EUR_USD".to_string())
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get price data");
    
    let response = result.unwrap();
    
    // Test that the response can be serialized back to JSON
    let json_result = serde_json::to_string(&response);
    assert!(json_result.is_ok(), "Response should be serializable to JSON");
    
    if let Ok(json_str) = json_result {
        assert!(!json_str.is_empty(), "JSON string should not be empty");
        assert!(json_str.contains("prices") || json_str.contains("null"), 
            "JSON should contain prices field or null");
    }
}

#[tokio::test]
async fn test_account_data_serialization() {
    let client = create_test_client();
    
    let result = ListAccountsRequest::new()
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get accounts");
    
    let accounts = result.unwrap();
    
    // Test serialization
    let json_result = serde_json::to_string(&accounts);
    assert!(json_result.is_ok(), "Accounts should be serializable");
    
    if let Ok(json_str) = json_result {
        assert!(!json_str.is_empty(), "JSON string should not be empty");
        assert!(json_str.contains("accounts"), "JSON should contain accounts field");
    }
}

#[tokio::test]
async fn test_candlestick_serialization() {
    let client = create_test_client();
    
    let result = GetInstrumentCandlesRequest::new()
        .with_instrument("EUR_USD".to_string())
        .with_granularity(CandlestickGranularity::H1)
        .with_count(5)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get candlestick data");
    
    let response = result.unwrap();
    
    // Test serialization
    let json_result = serde_json::to_string(&response);
    assert!(json_result.is_ok(), "Candlestick response should be serializable");
    
    if let Ok(json_str) = json_result {
        assert!(!json_str.is_empty(), "JSON string should not be empty");
        assert!(json_str.contains("instrument") || json_str.contains("candles"), 
            "JSON should contain instrument or candles data");
    }
}