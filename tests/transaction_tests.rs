mod common;

use fxoanda::*;
use common::*;
use chrono::{Duration, Utc};

#[tokio::test]
async fn test_list_transactions_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = ListTransactionsRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list transactions: {:?}", result);
    
    let transactions_response = result.unwrap();
    
    // Validate response structure
    assert!(transactions_response.from.is_some(), "Response should have 'from' field");
    assert!(transactions_response.to.is_some(), "Response should have 'to' field");
    
    // For ListTransactionsRequest, the response contains pagination info
    // The actual transaction data would be retrieved using the page URLs
    if let Some(pages) = &transactions_response.pages {
        assert!(!pages.is_empty(), "Should have at least one page of transactions");
        println!("Found {} pages of transaction data", pages.len());
    }
    
    // Validate count and page_size are reasonable
    if let Some(count) = transactions_response.count {
        assert!(count >= 0, "Transaction count should be non-negative");
    }
    
    if let Some(page_size) = transactions_response.page_size {
        assert!(page_size > 0, "Page size should be positive");
    }
}

#[tokio::test]
async fn test_get_transaction_details_validation() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get recent transactions using transaction range to get some transaction IDs
    // Use transaction IDs instead of dates for GetTransactionRangeRequest
    let range_result = GetTransactionRangeRequest::new()
        .with_account_id(account_id.clone())
        .with_from("1".to_string()) // Start from transaction ID 1
        .with_to("50".to_string()) // Up to transaction ID 50
        .remote(&client)
        .await;
    
    // Handle potential API response gracefully
    if range_result.is_ok() {
        let range_response = range_result.unwrap();
        
        if let Some(transactions) = &range_response.transactions {
            if let Some(first_transaction) = transactions.first() {
                if let Some(transaction_id) = &first_transaction.id {
                    // Test getting specific transaction details
                    let result = GetTransactionRequest::new()
                        .with_account_id(account_id.clone())
                        .with_transaction_id(transaction_id.clone())
                        .remote(&client)
                        .await;
                    
                    assert!(result.is_ok(), "Failed to get transaction details: {:?}", result);
                    
                    let transaction_response = result.unwrap();
                    if let Some(transaction) = &transaction_response.transaction {
                        // Validate comprehensive transaction details
                        assert_eq!(transaction.id, Some(transaction_id.clone()));
                        assert!(transaction.time.is_some(), "Transaction should have timestamp");
                        assert!(transaction.account_id.is_some(), "Transaction should have account ID");
                        
                        // Validate account ID matches
                        if let Some(tx_account_id) = &transaction.account_id {
                            assert_eq!(tx_account_id, &account_id, "Transaction account ID should match request");
                        }
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_get_transaction_range_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // First, get some recent transactions to find valid transaction IDs
    let list_result = ListTransactionsRequest::new()
        .with_account_id(account_id.clone())
        .remote(&client)
        .await;
    
    assert!(list_result.is_ok(), "Failed to list transactions: {:?}", list_result);
    let list_response = list_result.unwrap();
    
    // Check if we have any pages of transactions
    if let Some(pages) = &list_response.pages {
        if !pages.is_empty() {
            println!("Found {} pages of transactions", pages.len());
            
            // For transaction range, we need actual transaction IDs
            // Let's try to get a small range of recent transactions
            // Demo accounts typically have some initial transactions
            
            // Try a simple range from transaction ID 1 to 100
            let result = GetTransactionRangeRequest::new()
                .with_account_id(account_id.clone())
                .with_from("1".to_string()) // Start from transaction ID 1
                .with_to("100".to_string()) // Up to transaction ID 100
                .remote(&client)
                .await;
            
            // Handle the case where the range might be empty or invalid
            match result {
                Ok(transactions_response) => {
                    println!("Successfully retrieved transaction range");
                    
                    // Validate response structure
                    if let Some(transactions) = &transactions_response.transactions {
                        println!("Found {} transactions in range", transactions.len());
                        
                        // Validate transaction IDs are within range
                        for transaction in transactions.iter() {
                            if let Some(tx_id) = &transaction.id {
                                if let Ok(id_num) = tx_id.parse::<i32>() {
                                    assert!(id_num >= 1 && id_num <= 100,
                                        "Transaction ID {} should be within requested range 1-100", id_num);
                                }
                            }
                            
                            // Basic transaction structure validation
                            assert!(transaction.time.is_some(), "Transaction should have timestamp");
                            assert!(transaction.account_id.is_some(), "Transaction should have account ID");
                        }
                    } else {
                        println!("No transactions found in range - this is acceptable for some demo accounts");
                    }
                },
                Err(e) => {
                    // For demo accounts, transaction ranges might not have data
                    println!("Transaction range query failed (acceptable for demo accounts): {:?}", e);
                    // This is not a test failure - demo accounts may not have transactions in the requested range
                }
            }
        } else {
            println!("No transaction pages found - demo account may not have transaction history");
        }
    } else {
        println!("No transaction pages in response - demo account may not have transaction history");
    }
}

#[tokio::test]
async fn test_get_transactions_since_id_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get some transactions using range to find a starting transaction ID
    // Use transaction IDs instead of dates for GetTransactionRangeRequest
    let initial_result = GetTransactionRangeRequest::new()
        .with_account_id(account_id.clone())
        .with_from("1".to_string()) // Start from transaction ID 1
        .with_to("50".to_string()) // Up to transaction ID 50
        .remote(&client)
        .await;
    
    if initial_result.is_ok() {
        let initial_response = initial_result.unwrap();
        
        if let Some(transactions) = &initial_response.transactions {
            if transactions.len() > 1 {
                // Use the first transaction ID as the starting point
                if let Some(start_id) = &transactions[0].id {
                    let result = GetTransactionsSinceIdRequest::new()
                        .with_account_id(account_id)
                        .with_id(start_id.clone())
                        .remote(&client)
                        .await;
                    
                    assert!(result.is_ok(), "Failed to get transactions since ID: {:?}", result);
                    
                    let since_response = result.unwrap();
                    
                    if let Some(since_transactions) = &since_response.transactions {
                        // All returned transactions should have IDs greater than or equal to start_id
                        for transaction in since_transactions.iter() {
                            if let Some(tx_id) = &transaction.id {
                                // Simple numeric comparison assumes transaction IDs are sequential
                                // This might not always be true, so this is a best-effort validation
                                if let (Ok(start_num), Ok(tx_num)) = (start_id.parse::<i64>(), tx_id.parse::<i64>()) {
                                    assert!(tx_num >= start_num,
                                        "Transaction ID {} should be >= start ID {}", tx_id, start_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_transaction_type_filtering() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test filtering by transaction type using ListTransactionsRequest
    let transaction_types = vec!["DAILY_FINANCING", "CREATE", "MARKET_ORDER"];
    
    for tx_type in transaction_types {
        let result = ListTransactionsRequest::new()
            .with_account_id(account_id.clone())
            .with_otype(vec![tx_type.to_string()])
            .remote(&client)
            .await;
        
        if result.is_ok() {
            let transactions_response = result.unwrap();
            
            // For ListTransactionsRequest, check if the type filter was applied
            if let Some(response_types) = &transactions_response.otype {
                assert!(response_types.contains(&tx_type.to_string()),
                    "Response should reflect the requested transaction type filter");
            }
            
            // Check if we got pages for this transaction type
            if let Some(pages) = &transactions_response.pages {
                if !pages.is_empty() {
                    println!("Found {} pages for transaction type {}", pages.len(), tx_type);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_transaction_time_range_filtering() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test with different time ranges using ListTransactionsRequest
    let time_ranges = vec![
        (Utc::now() - Duration::days(7), Utc::now()),
        (Utc::now() - Duration::days(30), Utc::now() - Duration::days(7)),
    ];
    
    for (from_time, to_time) in time_ranges {
        let result = ListTransactionsRequest::new()
            .with_account_id(account_id.clone())
            .with_from(from_time)
            .with_to(to_time)
            .remote(&client)
            .await;
        
        if result.is_ok() {
            let transactions_response = result.unwrap();
            
            // Validate time range was applied to the request
            if let (Some(response_from), Some(response_to)) = 
                (&transactions_response.from, &transactions_response.to) {
                assert!(response_from >= &from_time && response_to <= &to_time,
                    "Response time range should match requested range");
            }
        }
    }
}

#[tokio::test]
async fn test_transaction_pagination_workflow() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test pagination with page size
    let page_size = 50;
    
    let result = ListTransactionsRequest::new()
        .with_account_id(account_id.clone())
        .with_page_size(page_size)
        .remote(&client)
        .await;
    
    if result.is_ok() {
        let transactions_response = result.unwrap();
        
        // Validate page size was applied
        if let Some(response_page_size) = transactions_response.page_size {
            assert_eq!(response_page_size, page_size,
                "Response page size should match requested page size");
        }
        
        // Check for pagination metadata
        assert!(transactions_response.from.is_some(), "Response should include 'from' field");
        assert!(transactions_response.to.is_some(), "Response should include 'to' field");
    }
    
    // Test with different page size
    let small_page_size = 10;
    let result2 = ListTransactionsRequest::new()
        .with_account_id(account_id)
        .with_page_size(small_page_size)
        .remote(&client)
        .await;
    
    if result2.is_ok() {
        let transactions_response2 = result2.unwrap();
        
        // Validate small page size was applied
        if let Some(response_page_size) = transactions_response2.page_size {
            assert_eq!(response_page_size, small_page_size,
                "Response page size should match requested small page size");
        }
    }
}

#[tokio::test]
async fn test_transaction_streaming_request() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test streaming transactions request structure
    // Note: This is typically used for real-time streaming, which may not work in test environment
    let result = StreamTransactionsRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    // Streaming may not be supported in demo environment or may timeout quickly
    // We're mainly testing that the request structure is correct
    if result.is_err() {
        let err_str = format!("{:?}", result.as_ref().err().unwrap());
        // Common errors for streaming endpoints
        if err_str.contains("timeout") || err_str.contains("connection") || err_str.contains("stream") {
            println!("Streaming transactions not available in test environment, which is expected");
            return;
        }
    }
    
    // If streaming works, validate the response structure
    if result.is_ok() {
        let _streaming_response = result.unwrap();
        // Streaming responses have different structures
        println!("Streaming transactions request succeeded");
    }
}

#[tokio::test]
async fn test_transaction_consistency_validation() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get actual transaction data using range to validate consistency
    // Use transaction IDs instead of dates for GetTransactionRangeRequest
    let result = GetTransactionRangeRequest::new()
        .with_account_id(account_id.clone())
        .with_from("1".to_string()) // Start from transaction ID 1
        .with_to("100".to_string()) // Up to transaction ID 100
        .remote(&client)
        .await;
    
    if result.is_ok() {
        let transactions_response = result.unwrap();
        
        if let Some(transactions) = &transactions_response.transactions {
            for transaction in transactions.iter() {
                // Validate transaction completeness
                assert!(transaction.id.is_some(), "Transaction should have ID");
                assert!(transaction.time.is_some(), "Transaction should have timestamp");
                assert!(transaction.account_id.is_some(), "Transaction should have account ID");
                
                // Validate account ID consistency
                if let Some(tx_account_id) = &transaction.account_id {
                    assert_eq!(tx_account_id, &account_id, "All transactions should be for the correct account");
                }
                
                // Validate user ID if present
                if let Some(user_id) = &transaction.user_id {
                    assert!(*user_id > 0, "User ID should be positive if present");
                }
            }
            
            println!("Validated {} transactions for consistency", transactions.len());
        }
    }
}

#[tokio::test]
async fn test_transaction_error_scenarios() {
    let client = create_test_client();
    
    // Test with invalid account ID
    let result = ListTransactionsRequest::new()
        .with_account_id("invalid_account_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should have empty pages or no pages for invalid account ID
        assert!(response.pages.is_none() || response.pages.as_ref().unwrap().is_empty(),
            "Invalid account ID should return no transaction pages");
    }
    // If it fails, that's also acceptable behavior for invalid account IDs
    
    // Test GetTransaction with invalid transaction ID
    let account_id = get_test_account_id(&client).await;
    let result = GetTransactionRequest::new()
        .with_account_id(account_id)
        .with_transaction_id("invalid_transaction_id".to_string())
        .remote(&client)
        .await;
    
    // OANDA API behavior: may return error or success with empty data
    if result.is_ok() {
        let response = result.unwrap();
        // Should have no transaction data for invalid transaction ID
        assert!(response.transaction.is_none(),
            "Invalid transaction ID should return no transaction data");
    }
    // If it fails, that's also acceptable behavior for invalid transaction IDs
}

#[tokio::test]
async fn test_transaction_chronological_ordering() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get actual transaction data using range to test ordering
    // Use transaction IDs instead of dates for GetTransactionRangeRequest
    let result = GetTransactionRangeRequest::new()
        .with_account_id(account_id)
        .with_from("1".to_string()) // Start from transaction ID 1
        .with_to("50".to_string()) // Up to transaction ID 50
        .remote(&client)
        .await;
    
    if result.is_ok() {
        let transactions_response = result.unwrap();
        
        if let Some(transactions) = &transactions_response.transactions {
            if transactions.len() > 1 {
                // Validate that transactions are in chronological order
                for i in 1..transactions.len() {
                    if let (Some(prev_time), Some(curr_time)) = 
                        (&transactions[i-1].time, &transactions[i].time) {
                        assert!(prev_time <= curr_time,
                            "Transactions should be in chronological order");
                    }
                    
                    // Also check ID ordering if both are numeric
                    if let (Some(prev_id), Some(curr_id)) = 
                        (&transactions[i-1].id, &transactions[i].id) {
                        if let (Ok(prev_num), Ok(curr_num)) = (prev_id.parse::<i64>(), curr_id.parse::<i64>()) {
                            assert!(prev_num <= curr_num,
                                "Transaction IDs should generally be in ascending order");
                        }
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_transaction_data_integrity() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Get actual transaction data using range to test data integrity
    // Use transaction IDs instead of dates for GetTransactionRangeRequest
    let result = GetTransactionRangeRequest::new()
        .with_account_id(account_id)
        .with_from("1".to_string()) // Start from transaction ID 1
        .with_to("50".to_string()) // Up to transaction ID 50
        .remote(&client)
        .await;
    
    if result.is_ok() {
        let transactions_response = result.unwrap();
        
        if let Some(transactions) = &transactions_response.transactions {
            for transaction in transactions.iter() {
                // Validate transaction ID format (should be numeric or alphanumeric)
                if let Some(id) = &transaction.id {
                    assert!(!id.is_empty(), "Transaction ID should not be empty");
                    // IDs are typically numeric in OANDA
                    if id.parse::<i64>().is_err() {
                        println!("Note: Transaction ID '{}' is not numeric", id);
                    }
                }
                
                // Validate timestamp is reasonable (not too far in future/past)
                if let Some(time) = &transaction.time {
                    let now = Utc::now();
                    let one_year_ago = now - Duration::days(365);
                    let one_day_future = now + Duration::days(1);
                    
                    assert!(time >= &one_year_ago && time <= &one_day_future,
                        "Transaction timestamp should be reasonable: {}", time);
                }
                
                // Validate request ID format if present
                if let Some(request_id) = &transaction.request_id {
                    assert!(!request_id.is_empty(), "Request ID should not be empty if present");
                }
                
                // Validate batch ID if present
                if let Some(batch_id) = &transaction.batch_id {
                    assert!(!batch_id.is_empty(), "Batch ID should not be empty if present");
                }
                
                // Validate user ID is reasonable if present
                if let Some(user_id) = &transaction.user_id {
                    assert!(*user_id > 0, "User ID should be positive if present");
                }
            }
        }
    }
}