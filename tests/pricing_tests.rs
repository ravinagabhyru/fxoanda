mod common;

use fxoanda::*;
use common::*;

#[tokio::test]
async fn test_get_current_prices() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = GetPricesRequest::new()
        .with_account_id(account_id)
        .with_instruments("EUR_USD".to_string())
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get current prices: {:?}", result);
    
    let response = result.unwrap();
    if let Some(prices) = &response.prices {
        assert!(!prices.is_empty(), "Should have price data");
        
        let price = &prices[0];
        assert!(price.instrument.is_some(), "Price should have instrument");
        assert!(price.time.is_some(), "Price should have timestamp");
        
        // Should have at least bid or ask prices
        let has_bids = price.bids.is_some() && !price.bids.as_ref().unwrap().is_empty();
        let has_asks = price.asks.is_some() && !price.asks.as_ref().unwrap().is_empty();
        
        assert!(has_bids || has_asks, "Price should have bid or ask data");
        
        if has_bids {
            let bid = &price.bids.as_ref().unwrap()[0];
            if let Some(bid_price) = bid.price {
                assert_price_precision(bid_price.into(), "EUR_USD");
            }
        }
    }
}

#[tokio::test]
async fn test_pricing_precision() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = GetPricesRequest::new()
        .with_account_id(account_id)
        .with_instruments("USD_JPY".to_string())
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get USD_JPY prices: {:?}", result);
    
    let response = result.unwrap();
    if let Some(prices) = &response.prices {
        if !prices.is_empty() {
            let price = &prices[0];
            if let Some(bids) = &price.bids {
                if !bids.is_empty() {
                    if let Some(bid_price) = bids[0].price {
                        assert_price_precision(bid_price.into(), "USD_JPY");
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_pricing_error_handling() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test with invalid instrument
    let result = GetPricesRequest::new()
        .with_account_id(account_id.clone())
        .with_instruments("INVALID_PAIR".to_string())
        .remote(&client)
        .await;
    
    assert!(result.is_err(), "Request with invalid instrument should fail");
    
    // Test with invalid account ID
    let result = GetPricesRequest::new()
        .with_account_id("invalid_account".to_string())
        .with_instruments("EUR_USD".to_string())
        .remote(&client)
        .await;
    
    assert!(result.is_err(), "Request with invalid account ID should fail");
}