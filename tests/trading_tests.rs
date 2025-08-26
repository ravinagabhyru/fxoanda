mod common;

use fxoanda::*;
use common::*;

#[tokio::test]
async fn test_list_all_orders() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListOrdersRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list orders: {:?}", result);
    
    let orders_response = result.unwrap();
    
    // Orders list might be empty for a fresh demo account, which is fine
    if let Some(orders) = &orders_response.orders {
        for order in orders.iter() {
            assert!(order.id.is_some(), "Order should have an ID");
            assert!(order.state.is_some(), "Order should have state");
        }
    }
}

#[tokio::test]
async fn test_list_pending_orders() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListPendingOrdersRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list pending orders: {:?}", result);
    
    let orders_response = result.unwrap();
    
    // Pending orders list might be empty for a fresh demo account
    if let Some(orders) = &orders_response.orders {
        for order in orders.iter() {
            // All returned orders should be pending
            if let Some(state) = &order.state {
                assert_ne!(state, "FILLED", "Pending orders should not be FILLED");
                assert_ne!(state, "CANCELLED", "Pending orders should not be CANCELLED");
            }
        }
    }
}

#[tokio::test]
async fn test_order_error_handling() {
    let client = create_test_client();
    
    // Test with invalid account ID for listing orders - API may return success with empty data
    let result = ListOrdersRequest::new()
        .with_account_id("invalid_account_id".to_string())
        .remote(&client)
        .await;
    
    // API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should have no orders for invalid account ID
        assert!(response.orders.is_none() || response.orders.as_ref().unwrap().is_empty(),
            "Invalid account ID should return no orders");
    }
    // If it fails, that's also acceptable behavior for invalid account IDs
}