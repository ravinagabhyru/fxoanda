mod common;

use fxoanda::*;
use common::*;

#[tokio::test]
async fn test_list_all_positions() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListPositionsRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list positions: {:?}", result);
    
    let positions_response = result.unwrap();
    
    // Positions might be empty for a fresh demo account
    if let Some(positions) = &positions_response.positions {
        for position in positions.iter() {
            assert!(position.instrument.is_some(), "Position should have instrument");
            
            // Validate position structure
            if let Some(long) = &position.long {
                if let Some(units) = long.units {
                    assert!(units >= 0.0, "Long units should not be negative");
                }
            }
            
            if let Some(short) = &position.short {
                if let Some(units) = short.units {
                    assert!(units <= 0.0, "Short units should not be positive");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_list_open_positions() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListOpenPositionsRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list open positions: {:?}", result);
    
    let positions_response = result.unwrap();
    
    // Open positions might be empty for a fresh demo account
    if let Some(positions) = &positions_response.positions {
        for position in positions.iter() {
            // All returned positions should be open (have non-zero units)
            let has_long_units = position.long.as_ref()
                .and_then(|l| l.units)
                .map(|u| u != 0.0)
                .unwrap_or(false);
            
            let has_short_units = position.short.as_ref()
                .and_then(|s| s.units)
                .map(|u| u != 0.0)
                .unwrap_or(false);
            
            assert!(
                has_long_units || has_short_units,
                "Open position should have non-zero units"
            );
        }
    }
}

#[tokio::test]
async fn test_get_position_details() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test getting position details for EUR_USD
    let result = GetPositionRequest::new()
        .with_account_id(account_id)
        .with_instrument("EUR_USD".to_string())
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get position details: {:?}", result);
    
    let position_response = result.unwrap();
    if let Some(position) = &position_response.position {
        assert_eq!(position.instrument, Some("EUR_USD".to_string()));
        
        // Validate position structure
        if let Some(long) = &position.long {
            assert!(long.units.is_some(), "Long position should have units field");
            if let Some(unrealized_pl) = long.unrealized_pl {
                // P&L can be positive or negative
                assert!(unrealized_pl.is_finite(), "Unrealized P&L should be a valid number");
            }
        }
        
        if let Some(short) = &position.short {
            assert!(short.units.is_some(), "Short position should have units field");
        }
    }
}

#[tokio::test]
async fn test_position_error_handling() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test with invalid instrument - API may return success with empty position data
    let result = GetPositionRequest::new()
        .with_account_id(account_id.clone())
        .with_instrument("INVALID_PAIR".to_string())
        .remote(&client)
        .await;
    
    // API behavior: may return error or success with empty data  
    if result.is_ok() {
        let response = result.unwrap();
        // Should have no position data for invalid instrument
        assert!(response.position.is_none(),
            "Invalid instrument should return no position data");
    }
    // If it fails, that's also acceptable behavior for invalid instruments
    
    // Test with invalid account ID - API may return success with empty data
    let result = GetPositionRequest::new()
        .with_account_id("invalid_account".to_string())
        .with_instrument("EUR_USD".to_string())
        .remote(&client)
        .await;
    
    // API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should have no position data for invalid account ID
        assert!(response.position.is_none(),
            "Invalid account ID should return no position data");
    }
    // If it fails, that's also acceptable behavior for invalid account IDs
}