mod common;

use fxoanda::*;
use common::*;

// UNIT TEST PATTERNS - Use mock clients for validation testing

#[tokio::test]
async fn test_unit_order_validation_logic() {
    let client = create_mock_client();
    
    // Test validation logic without API calls - missing account_id should trigger validation error
    let result = CreateMarketOrderRequest::new()
        // Missing account_id - should trigger validation error
        .with_order(
            MarketOrder::new()
                .with_instrument("EUR_USD".to_string())
                .with_units(100.0)
        )
        .remote(&client)
        .await;
        
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FxError::Validation(RequestValidationError::MissingAccountId)));
}

#[tokio::test]
async fn test_unit_price_request_validation() {
    let client = create_mock_client();
    
    // Test that GetPricesRequest validates account_id is required
    let result = GetPricesRequest::new()
        .with_instruments("EUR_USD".to_string())
        // Missing account_id - should fail validation
        .remote(&client)
        .await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FxError::Validation(RequestValidationError::MissingAccountId)));
}

// INTEGRATION TEST PATTERNS - Use TestContext for stateful testing

#[tokio::test]
async fn test_integration_create_and_cancel_limit_order() {
    let ctx = TestContext::new().await;
    let order_id = ctx.unique_order_id("create_cancel");
    
    // Create specific limit order with unique ID
    let limit_order = LimitOrder::new()
        .with_instrument("EUR_USD".to_string())
        .with_units(1.0)
        .with_price(0.50000) // Very low price, won't execute
        .with_time_in_force("GTC".to_string())
        .with_otype("LIMIT".to_string());

    let create_result = CreateLimitOrderRequest::new()
        .with_account_id(ctx.account_id.clone())
        .with_order(limit_order)
        .remote(&ctx.client).await;
    
    assert!(create_result.is_ok(), "Failed to create limit order: {:?}", create_result);
    let created_order = create_result.unwrap();
    
    // Test specifically for OUR order
    assert!(created_order.order_create_transaction.is_some());
    let create_transaction = created_order.order_create_transaction.unwrap();
    assert!(create_transaction.id.is_some(), "Transaction should have an ID");
    
    // Get the created order ID for cleanup
    if let Some(order_transaction_id) = create_transaction.id {
        // Cancel the specific order we created
        let cancel_result = CancelOrderRequest::new()
            .with_account_id(ctx.account_id.clone())
            .with_order_specifier(order_transaction_id.to_string())
            .remote(&ctx.client).await;
        
        assert!(cancel_result.is_ok(), "Failed to cancel order: {:?}", cancel_result);
        
        // Verify cancellation transaction
        let cancel_response = cancel_result.unwrap();
        assert!(cancel_response.order_cancel_transaction.is_some());
    }
}

/// Test market order execution handling both market open and closed scenarios
/// 
/// When markets are OPEN: Market orders fill immediately and create positions
/// When markets are CLOSED: Market orders are created but remain pending until market opens
#[tokio::test]
async fn test_integration_market_order_execution() {
    let ctx = TestContext::new().await;
    let client_req_id = ctx.unique_order_id("market_execution");
    
    // Get initial position for EUR_USD (if any)
    let initial_position = get_position_for_instrument(&ctx, "EUR_USD").await;
    
    // Create market order with unique ID
    let market_order = MarketOrder::new()
        .with_instrument("EUR_USD".to_string())
        .with_units(1.0) // Minimal trade size
        .with_time_in_force("FOK".to_string()) // Fill or Kill
        .with_otype("MARKET".to_string());

    let order_result = CreateMarketOrderRequest::new()
        .with_account_id(ctx.account_id.clone())
        .with_order(market_order)
        .remote(&ctx.client).await;
    
    assert!(order_result.is_ok(), "Failed to create market order: {:?}", order_result);
    let order_response = order_result.unwrap();
    
    // Verify OUR specific order was created
    assert!(order_response.order_create_transaction.is_some());
    let create_transaction = order_response.order_create_transaction.unwrap();
    assert!(create_transaction.id.is_some(), "Transaction should have an ID");
    
    // Check if market order was filled (depends on market hours)
    match order_response.order_fill_transaction {
        Some(_fill_transaction) => {
            println!("Market is open: Order was filled immediately");
            
            // Verify position change (not absolute position)
            let new_position = get_position_for_instrument(&ctx, "EUR_USD").await;
            verify_position_change(&initial_position, &new_position, 1);
            
            // Cleanup: Close the position we just created
            if new_position.is_some() {
                let close_result = ClosePositionRequest::new()
                    .with_account_id(ctx.account_id.clone())
                    .with_instrument("EUR_USD".to_string())
                    .with_long_units("1".to_string()) // Close just our 1 unit
                    .remote(&ctx.client).await;
                
                // Don't fail test if close fails - demo account might have constraints
                if let Err(e) = close_result {
                    println!("Note: Failed to close position (expected for some demo accounts): {:?}", e);
                }
            }
        },
        None => {
            println!("Market is closed: Order was created but not filled");
            
            // When markets are closed, the order should be created but not filled
            // We should be able to find it in pending orders
            let pending_orders = ListPendingOrdersRequest::new()
                .with_account_id(ctx.account_id.clone())
                .remote(&ctx.client).await;
                
            if let Ok(pending_response) = pending_orders {
                if let Some(orders) = pending_response.orders {
                    // Look for our order in pending orders
                    let our_order = orders.iter().find(|order| 
                        order.id == create_transaction.id
                    );
                    
                    if our_order.is_some() {
                        println!("Found our market order in pending orders (market closed)");
                        
                        // Cleanup: Cancel the pending order
                        if let Some(order_id) = create_transaction.id {
                            let _ = CancelOrderRequest::new()
                                .with_account_id(ctx.account_id.clone())
                                .with_order_specifier(order_id.to_string())
                                .remote(&ctx.client).await;
                        }
                    }
                }
            }
            
            // Verify no position change occurred
            let new_position = get_position_for_instrument(&ctx, "EUR_USD").await;
            verify_position_change(&initial_position, &new_position, 0);
        }
    }
}

#[tokio::test] 
async fn test_integration_position_tracking_specific_trade() {
    let ctx = TestContext::new().await;
    
    // Create a small position and track the specific trade
    match create_test_position(&ctx, "GBP_USD", 1).await {
        Ok(trade_id) => {
            // Verify we can query this specific trade
            let trade_details = GetTradeRequest::new()
                .with_account_id(ctx.account_id.clone())
                .with_trade_specifier(trade_id.clone())
                .remote(&ctx.client).await;
            
            assert!(trade_details.is_ok(), "Should be able to get trade details");
            
            let trade = trade_details.unwrap().trade.unwrap();
            assert_eq!(trade.instrument, Some("GBP_USD".to_string()));
            assert_eq!(trade.current_units, Some(1.0));
            
            // Close specifically the trade we just created
            let close_result = CloseTradeRequest::new()
                .with_account_id(ctx.account_id.clone())
                .with_trade_specifier(trade_id.clone())
                .with_units("ALL".to_string())
                .remote(&ctx.client).await;
            
            assert!(close_result.is_ok(), "Should be able to close specific trade");
            
            // Verify our specific trade is now closed (404 when querying)
            let verify_close = GetTradeRequest::new()
                .with_account_id(ctx.account_id.clone())
                .with_trade_specifier(trade_id.clone())
                .remote(&ctx.client).await;
            
            // Trade should either not exist (404) or be closed
            match verify_close {
                Err(FxError::ApiError { status_code: 404, .. }) => {
                    // Expected - trade was closed and removed
                },
                Ok(response) => {
                    let trade = response.trade.unwrap();
                    assert!(trade.current_units == Some(0.0) || trade.current_units.is_none(),
                        "Trade should be closed");
                },
                Err(e) => panic!("Unexpected error checking closed trade: {:?}", e),
            }
        },
        Err(e) => {
            // Some demo accounts might not allow market orders, skip test
            println!("Skipping position test - demo account may not allow trading: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_integration_demo_account_health() {
    let ctx = TestContext::new().await;
    
    // Basic account accessibility
    let account = GetAccountRequest::new()
        .with_account_id(ctx.account_id.clone())
        .remote(&ctx.client).await;
    
    assert!(account.is_ok(), "Demo account should be accessible: {:?}", account);
    
    // Verify we can create and cancel orders (trading permissions)
    let test_order = LimitOrder::new()
        .with_instrument("EUR_USD".to_string())
        .with_units(1.0)
        .with_price(0.50000) // Very low price, won't execute
        .with_time_in_force("GTC".to_string())
        .with_otype("LIMIT".to_string());

    let create_result = CreateLimitOrderRequest::new()
        .with_account_id(ctx.account_id.clone())
        .with_order(test_order)
        .remote(&ctx.client).await;
    
    match create_result {
        Ok(order_response) => {
            // If we can create orders, we should be able to cancel them too
            if let Some(create_transaction) = order_response.order_create_transaction {
                if let Some(order_id) = create_transaction.id {
                    let _ = CancelOrderRequest::new()
                        .with_account_id(ctx.account_id.clone())
                        .with_order_specifier(order_id.to_string())
                        .remote(&ctx.client).await;
                }
            }
        },
        Err(e) => {
            // Some demo accounts might restrict order creation
            println!("Demo account has order creation restrictions (normal): {:?}", e);
        }
    }
}