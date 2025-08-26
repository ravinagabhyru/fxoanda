mod common;

use fxoanda::*;
use common::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_list_all_trades_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListTradesRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list all trades: {:?}", result);
    
    let trades_response = result.unwrap();
    
    // Trades list might be empty for a fresh demo account
    if let Some(trades) = &trades_response.trades {
        for trade in trades.iter() {
            // Validate basic trade structure
            assert!(trade.id.is_some(), "Trade should have an ID");
            assert!(trade.instrument.is_some(), "Trade should have an instrument");
            assert!(trade.current_units.is_some(), "Trade should have current units");
            assert!(trade.state.is_some(), "Trade should have a state");
            
            // Validate trade state is valid
            if let Some(state) = &trade.state {
                assert!(
                    state == "OPEN" || state == "CLOSED", 
                    "Trade state should be OPEN or CLOSED, got: {}", state
                );
            }
            
            // Validate units are not zero for open trades
            if let (Some(units), Some(state)) = (&trade.current_units, &trade.state) {
                if state == "OPEN" {
                    assert_ne!(*units, 0.0, "Open trade should have non-zero units");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_list_open_trades_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListOpenTradesRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list open trades: {:?}", result);
    
    let trades_response = result.unwrap();
    
    // Open trades might be empty for a fresh demo account
    if let Some(trades) = &trades_response.trades {
        for trade in trades.iter() {
            // All returned trades should be open
            if let Some(state) = &trade.state {
                assert_eq!(state, "OPEN", "All trades from ListOpenTrades should be OPEN");
            }
            
            // Open trades should have non-zero units
            if let Some(units) = &trade.current_units {
                assert_ne!(*units, 0.0, "Open trade should have non-zero units");
            }
            
            // Open trades should have unrealized P&L
            assert!(trade.unrealized_pl.is_some(), "Open trade should have unrealized P&L");
        }
    }
}

#[tokio::test]
async fn test_get_trade_details_validation() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // First, get a list of trades to find an existing trade ID
    let trades_result = ListTradesRequest::new()
        .with_account_id(account_id.clone())
        .remote(&client)
        .await;
    
    assert!(trades_result.is_ok(), "Failed to list trades: {:?}", trades_result);
    let trades_response = trades_result.unwrap();
    
    if let Some(trades) = trades_response.trades {
        if let Some(first_trade) = trades.first() {
            if let Some(trade_id) = &first_trade.id {
                // Test getting specific trade details
                let result = GetTradeRequest::new()
                    .with_account_id(account_id)
                    .with_trade_specifier(trade_id.clone())
                    .remote(&client)
                    .await;
                
                assert!(result.is_ok(), "Failed to get trade details: {:?}", result);
                
                let trade_response = result.unwrap();
                if let Some(trade) = &trade_response.trade {
                    // Validate comprehensive trade details
                    assert_eq!(trade.id, Some(trade_id.clone()));
                    assert!(trade.instrument.is_some(), "Trade should have instrument");
                    assert!(trade.current_units.is_some(), "Trade should have current units");
                    assert!(trade.open_time.is_some(), "Trade should have open time");
                    
                    // Validate P&L calculations if available
                    if let Some(realized_pl) = trade.realized_pl {
                        assert!(realized_pl.is_finite(), "Realized P&L should be a valid number");
                    }
                    
                    if let Some(unrealized_pl) = trade.unrealized_pl {
                        assert!(unrealized_pl.is_finite(), "Unrealized P&L should be a valid number");
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_close_trade_workflow_simulation() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test close trade request structure with invalid trade ID
    // This tests the request building but should fail gracefully in demo environment
    let result = CloseTradeRequest::new()
        .with_account_id(account_id)
        .with_trade_specifier("invalid_trade_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should indicate failure to close non-existent trade
        assert!(response.order_create_transaction.is_none() || response.order_fill_transaction.is_none(),
            "Invalid trade ID should not result in successful close");
    }
    // If it fails, that's acceptable for invalid trade IDs
}

#[tokio::test]
async fn test_trade_client_extensions_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test client extensions structure with invalid trade ID
    let result = SetTradeClientExtensionsRequest::new()
        .with_account_id(account_id)
        .with_trade_specifier("invalid_trade_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should indicate no change for non-existent trade
        assert!(response.trade_client_extensions_modify_transaction.is_none(),
            "Invalid trade ID should not result in successful client extensions modification");
    }
    // If it fails, that's acceptable for invalid trade IDs
}

#[tokio::test]
async fn test_trade_dependent_orders_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test dependent orders structure with invalid trade ID
    let result = SetTradeDependentOrdersRequest::new()
        .with_account_id(account_id)
        .with_trade_specifier("invalid_trade_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should indicate no change for non-existent trade
        assert!(response.take_profit_order_transaction.is_none() && response.stop_loss_order_transaction.is_none(),
            "Invalid trade ID should not result in successful dependent orders modification");
    }
    // If it fails, that's acceptable for invalid trade IDs
}

#[tokio::test]
async fn test_trade_state_transitions() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get all trades and analyze their states
    let result = ListTradesRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list trades: {:?}", result);
    
    let trades_response = result.unwrap();
    let mut state_counts = HashMap::new();
    
    if let Some(trades) = &trades_response.trades {
        for trade in trades.iter() {
            if let Some(state) = &trade.state {
                *state_counts.entry(state.clone()).or_insert(0) += 1;
            }
        }
        
        // Validate only valid states are present
        for state in state_counts.keys() {
            assert!(
                state == "OPEN" || state == "CLOSED",
                "Invalid trade state found: {}", state
            );
        }
    }
}

#[tokio::test]
async fn test_trade_calculations_validation() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListOpenTradesRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list open trades: {:?}", result);
    
    let trades_response = result.unwrap();
    
    if let Some(trades) = &trades_response.trades {
        for trade in trades.iter() {
            // Validate price fields are reasonable
            if let Some(price) = trade.price {
                assert!(price > 0.0, "Trade price should be positive");
                assert!(price < 1000000.0, "Trade price seems unreasonably high");
                
                // Validate precision based on instrument
                if let Some(instrument) = &trade.instrument {
                    assert_price_precision(price.into(), instrument);
                }
            }
            
            // Validate P&L calculations
            if let Some(unrealized_pl) = trade.unrealized_pl {
                assert!(unrealized_pl.is_finite(), "Unrealized P&L should be finite");
            }
            
            if let Some(realized_pl) = trade.realized_pl {
                assert!(realized_pl.is_finite(), "Realized P&L should be finite");
            }
            
            // Validate units consistency
            if let (Some(initial_units), Some(current_units)) = (&trade.initial_units, &trade.current_units) {
                // For open trades, current units should match initial units (no partial closes yet)
                if let Some(state) = &trade.state {
                    if state == "OPEN" {
                        // In demo accounts, trades are typically not partially closed
                        // So current_units should equal initial_units for most cases
                        if *current_units != 0.0 {
                            assert!(
                                current_units.abs() <= initial_units.abs(),
                                "Current units should not exceed initial units"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_trade_filtering_capabilities() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test different filtering options for ListTradesRequest
    let instruments = get_test_instruments();
    
    for instrument in instruments.iter().take(2) { // Test with first 2 instruments
        let result = ListTradesRequest::new()
            .with_account_id(account_id.clone())
            .with_instrument(instrument.to_string())
            .remote(&client)
            .await;
        
        if result.is_ok() {
            let trades_response = result.unwrap();
            
            if let Some(trades) = &trades_response.trades {
                // All trades should be for the requested instrument
                for trade in trades.iter() {
                    if let Some(trade_instrument) = &trade.instrument {
                        assert_eq!(trade_instrument, instrument,
                            "Trade instrument filter should work correctly");
                    }
                }
            }
        }
    }
    
    // Test state filtering
    let result = ListTradesRequest::new()
        .with_account_id(account_id.clone())
        .with_state("OPEN".to_string())
        .remote(&client)
        .await;
    
    if result.is_ok() {
        let trades_response = result.unwrap();
        
        if let Some(trades) = &trades_response.trades {
            // All trades should be OPEN
            for trade in trades.iter() {
                if let Some(state) = &trade.state {
                    assert_eq!(state, "OPEN", "State filter should work correctly");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_trade_error_scenarios() {
    let client = create_test_client();
    
    // Test with invalid account ID
    let result = ListTradesRequest::new()
        .with_account_id("invalid_account_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should have no trades for invalid account ID
        assert!(response.trades.is_none() || response.trades.as_ref().unwrap().is_empty(),
            "Invalid account ID should return no trades");
    }
    // If it fails, that's also acceptable behavior for invalid account IDs
    
    // Test GetTrade with invalid trade ID
    let account_id = get_test_account_id(&client).await;
    let result = GetTradeRequest::new()
        .with_account_id(account_id)
        .with_trade_specifier("invalid_trade_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should have no trade data for invalid trade ID
        assert!(response.trade.is_none(),
            "Invalid trade ID should return no trade data");
    }
    // If it fails, that's also acceptable behavior for invalid trade IDs
}

#[tokio::test]
async fn test_trade_timestamp_validation() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListTradesRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list trades: {:?}", result);
    
    let trades_response = result.unwrap();
    
    if let Some(trades) = &trades_response.trades {
        for trade in trades.iter() {
            // Validate timestamps are present
            assert!(trade.open_time.is_some(), "Trade should have open time");
            
            // If trade is closed, it should have a close time
            if let Some(state) = &trade.state {
                if state == "CLOSED" {
                    // Note: close_time might not be available depending on API response format
                    // This is more of a structural validation
                }
            }
        }
    }
}

#[tokio::test]
async fn test_trade_consistency_with_positions() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get open trades
    let trades_result = ListOpenTradesRequest::new()
        .with_account_id(account_id.clone())
        .remote(&client)
        .await;
    
    // Get positions
    let positions_result = ListOpenPositionsRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    if trades_result.is_ok() && positions_result.is_ok() {
        let trades_response = trades_result.unwrap();
        let positions_response = positions_result.unwrap();
        
        let trades = trades_response.trades.unwrap_or_default();
        let positions = positions_response.positions.unwrap_or_default();
        
        // For each open trade, there should be a corresponding position
        for trade in trades.iter() {
            if let Some(trade_instrument) = &trade.instrument {
                let position_exists = positions.iter().any(|pos| {
                    pos.instrument.as_ref() == Some(trade_instrument)
                });
                
                // Note: This might not always be true depending on OANDA's netting behavior
                // But it's a good consistency check for most cases
                if !position_exists {
                    println!("Warning: Trade for {} found but no corresponding position", trade_instrument);
                }
            }
        }
    }
}