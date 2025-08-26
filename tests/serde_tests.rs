mod common;

use fxoanda::*;
use common::*;
use serde_json;
use chrono::prelude::*;
use fxoanda_serdes::{serfloats, serdates};

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

#[test]
fn test_custom_float_serialization() {
    use serde::{Deserialize, Serialize};
    use serde_json;
    
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestFloatStruct {
        #[serde(with = "serfloats")]
        price: Option<f32>,
        #[serde(with = "serfloats")]
        amount: Option<f32>,
    }
    
    // Test serialization with valid floats
    let test_data = TestFloatStruct {
        price: Some(1.23456),
        amount: Some(1000.0),
    };
    
    let json_result = serde_json::to_string(&test_data);
    assert!(json_result.is_ok(), "Should serialize float data successfully");
    
    let json_str = json_result.unwrap();
    println!("Serialized float data: {}", json_str);
    
    // Verify the floats are serialized as strings
    assert!(json_str.contains("\"1.23456\""), "Price should be serialized as string");
    assert!(json_str.contains("\"1000\""), "Amount should be serialized as string");
    
    // Test deserialization back from JSON
    let deserialized_result: Result<TestFloatStruct, _> = serde_json::from_str(&json_str);
    assert!(deserialized_result.is_ok(), "Should deserialize float data successfully");
    
    let deserialized_data = deserialized_result.unwrap();
    assert_eq!(deserialized_data.price, test_data.price, "Price should match after round-trip");
    assert_eq!(deserialized_data.amount, test_data.amount, "Amount should match after round-trip");
    
    // Test with None values
    let test_none = TestFloatStruct {
        price: None,
        amount: None,
    };
    
    let json_none = serde_json::to_string(&test_none).unwrap();
    println!("Serialized None float data: {}", json_none);
    
    let deserialized_none: TestFloatStruct = serde_json::from_str(&json_none).unwrap();
    assert_eq!(deserialized_none.price, None, "None price should remain None");
    assert_eq!(deserialized_none.amount, None, "None amount should remain None");
    
    // Test deserialization from string representation
    let json_from_string = r#"{"price":"0.00001","amount":"999999.99"}"#;
    let from_string: TestFloatStruct = serde_json::from_str(json_from_string).unwrap();
    assert_eq!(from_string.price, Some(0.00001), "Should parse small float from string");
    assert_eq!(from_string.amount, Some(999999.99), "Should parse large float from string");
}

#[test]
fn test_custom_date_serialization() {
    use serde::{Deserialize, Serialize};
    use serde_json;
    
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestDateStruct {
        #[serde(with = "serdates")]
        created_time: Option<DateTime<Utc>>,
        #[serde(with = "serdates")]
        updated_time: Option<DateTime<Utc>>,
    }
    
    // Create test datetime
    let test_time = Utc::now();
    let test_data = TestDateStruct {
        created_time: Some(test_time),
        updated_time: Some(test_time),
    };
    
    // Test serialization
    let json_result = serde_json::to_string(&test_data);
    assert!(json_result.is_ok(), "Should serialize datetime data successfully");
    
    let json_str = json_result.unwrap();
    println!("Serialized datetime data: {}", json_str);
    
    // Verify the datetime is serialized in RFC3339 format
    let expected_time_str = test_time.to_rfc3339();
    assert!(json_str.contains(&expected_time_str), "Should contain RFC3339 formatted timestamp");
    
    // Test deserialization back from JSON
    let deserialized_result: Result<TestDateStruct, _> = serde_json::from_str(&json_str);
    assert!(deserialized_result.is_ok(), "Should deserialize datetime data successfully");
    
    let deserialized_data = deserialized_result.unwrap();
    assert_eq!(deserialized_data.created_time, test_data.created_time, "Created time should match after round-trip");
    assert_eq!(deserialized_data.updated_time, test_data.updated_time, "Updated time should match after round-trip");
    
    // Test with None values
    let test_none = TestDateStruct {
        created_time: None,
        updated_time: None,
    };
    
    let json_none = serde_json::to_string(&test_none).unwrap();
    println!("Serialized None datetime data: {}", json_none);
    
    let deserialized_none: TestDateStruct = serde_json::from_str(&json_none).unwrap();
    assert_eq!(deserialized_none.created_time, None, "None created_time should remain None");
    assert_eq!(deserialized_none.updated_time, None, "None updated_time should remain None");
    
    // Test deserialization from RFC3339 string
    let rfc3339_json = r#"{"created_time":"2023-12-01T15:30:45.123456789Z","updated_time":"2024-01-15T09:45:30Z"}"#;
    let from_rfc3339: TestDateStruct = serde_json::from_str(rfc3339_json).unwrap();
    assert!(from_rfc3339.created_time.is_some(), "Should parse RFC3339 datetime with nanoseconds");
    assert!(from_rfc3339.updated_time.is_some(), "Should parse RFC3339 datetime without nanoseconds");
    
    // Test special case: "0" should deserialize to None
    let zero_json = r#"{"created_time":"0","updated_time":"0"}"#;
    let from_zero: TestDateStruct = serde_json::from_str(zero_json).unwrap();
    assert_eq!(from_zero.created_time, None, "String \"0\" should deserialize to None");
    assert_eq!(from_zero.updated_time, None, "String \"0\" should deserialize to None");
}

#[test]
fn test_financial_precision_validation() {
    use serde::{Deserialize, Serialize};
    use serde_json;
    
    #[derive(Serialize, Deserialize, Debug)]
    struct FinancialData {
        #[serde(with = "serfloats")]
        bid_price: Option<f32>,
        #[serde(with = "serfloats")]
        ask_price: Option<f32>,
        #[serde(with = "serfloats")]
        units: Option<f32>,
    }
    
    // Test with typical forex precision (5 decimal places for EUR/USD)
    let forex_data = FinancialData {
        bid_price: Some(1.18945),
        ask_price: Some(1.18955),
        units: Some(100000.0),
    };
    
    // Serialize and verify precision is maintained in string format
    let json_str = serde_json::to_string(&forex_data).unwrap();
    println!("Financial data JSON: {}", json_str);
    
    // Deserialize and validate precision
    let deserialized: FinancialData = serde_json::from_str(&json_str).unwrap();
    
    // Verify precision is maintained (within float32 accuracy)
    if let (Some(orig_bid), Some(deser_bid)) = (forex_data.bid_price, deserialized.bid_price) {
        let diff = (orig_bid - deser_bid).abs();
        assert!(diff < 0.00001, "Bid price precision should be maintained within float32 limits");
    }
    
    if let (Some(orig_ask), Some(deser_ask)) = (forex_data.ask_price, deserialized.ask_price) {
        let diff = (orig_ask - deser_ask).abs();
        assert!(diff < 0.00001, "Ask price precision should be maintained within float32 limits");
    }
    
    // Test edge cases for financial data
    let edge_cases = FinancialData {
        bid_price: Some(0.00001), // Very small price
        ask_price: Some(999999.99999), // Large price with decimals
        units: Some(-50000.0), // Negative units (short position)
    };
    
    let edge_json = serde_json::to_string(&edge_cases).unwrap();
    let edge_deserialized: FinancialData = serde_json::from_str(&edge_json).unwrap();
    
    assert!(edge_deserialized.bid_price.is_some(), "Should handle very small prices");
    assert!(edge_deserialized.ask_price.is_some(), "Should handle large prices with decimals");
    assert!(edge_deserialized.units.is_some(), "Should handle negative units");
    
    println!("Edge case validation completed successfully");
}

#[tokio::test]
async fn test_real_api_data_serialization_roundtrip() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test with real pricing data that uses custom serializers
    let pricing_result = GetPricesRequest::new()
        .with_account_id(account_id.clone())
        .with_instruments("EUR_USD,GBP_USD".to_string())
        .remote(&client)
        .await;
    
    if pricing_result.is_ok() {
        let pricing_response = pricing_result.unwrap();
        
        // Serialize the real response
        let serialized_json = serde_json::to_string(&pricing_response);
        assert!(serialized_json.is_ok(), "Real pricing response should serialize successfully");
        
        let json_str = serialized_json.unwrap();
        println!("Real pricing data serialized length: {}", json_str.len());
        
        // Deserialize back and verify structure integrity
        let deserialized_result: Result<GetPricesResponse, _> = serde_json::from_str(&json_str);
        assert!(deserialized_result.is_ok(), "Should deserialize back to original structure");
        
        let deserialized_response = deserialized_result.unwrap();
        
        // Verify key fields are preserved
        if let (Some(orig_prices), Some(deser_prices)) = (&pricing_response.prices, &deserialized_response.prices) {
            assert_eq!(orig_prices.len(), deser_prices.len(), "Price array length should be preserved");
            
            for (orig_price, deser_price) in orig_prices.iter().zip(deser_prices.iter()) {
                assert_eq!(orig_price.instrument, deser_price.instrument, "Instrument should be preserved");
                
                // Test bid/ask prices if present (these use custom float serialization)
                if let (Some(orig_bids), Some(deser_bids)) = (&orig_price.bids, &deser_price.bids) {
                    assert_eq!(orig_bids.len(), deser_bids.len(), "Bid buckets count should be preserved");
                    for (orig_bid, deser_bid) in orig_bids.iter().zip(deser_bids.iter()) {
                        if let (Some(orig_price), Some(deser_price)) = (orig_bid.price, deser_bid.price) {
                            let diff = (orig_price - deser_price).abs();
                            assert!(diff < 0.0001, "Bid price should be preserved through serialization");
                        }
                    }
                }
                
                if let (Some(orig_asks), Some(deser_asks)) = (&orig_price.asks, &deser_price.asks) {
                    assert_eq!(orig_asks.len(), deser_asks.len(), "Ask buckets count should be preserved");
                    for (orig_ask, deser_ask) in orig_asks.iter().zip(deser_asks.iter()) {
                        if let (Some(orig_price), Some(deser_price)) = (orig_ask.price, deser_ask.price) {
                            let diff = (orig_price - deser_price).abs();
                            assert!(diff < 0.0001, "Ask price should be preserved through serialization");
                        }
                    }
                }
                
                // Test timestamp preservation (uses custom date serialization)
                if let (Some(orig_time), Some(deser_time)) = (&orig_price.time, &deser_price.time) {
                    assert_eq!(orig_time, deser_time, "Timestamp should be preserved exactly");
                }
            }
        }
    }
    
    // Test with account data that may contain custom serialized fields
    let account_result = GetAccountRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    if account_result.is_ok() {
        let account_response = account_result.unwrap();
        
        let account_json = serde_json::to_string(&account_response);
        assert!(account_json.is_ok(), "Account response should serialize successfully");
        
        let account_json_str = account_json.unwrap();
        let account_deserialized: Result<GetAccountResponse, _> = serde_json::from_str(&account_json_str);
        assert!(account_deserialized.is_ok(), "Account data should deserialize successfully");
        
        println!("Real API data serialization roundtrip validation completed");
    }
}