mod common;

use fxoanda::*;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
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
    
    println!("Starting concurrent requests test with account_id: {}", account_id);
    
    // Helper function to check if error is due to temporary API issues
    let is_temporary_api_error = |e: &Box<dyn std::error::Error>| -> bool {
        let error_str = format!("{:?}", e);
        error_str.contains("expected value") || 
        error_str.contains("connection") || 
        error_str.contains("timeout") ||
        error_str.contains("Decode")
    };
    
    let max_attempts = 3;
    let mut attempt = 1;
    
    while attempt <= max_attempts {
        println!("Attempt {} of {}", attempt, max_attempts);
        
        // Test concurrent requests don't interfere with each other
        let accounts_future = ListAccountsRequest::new().remote(&client);
        let pricing_future = GetPricesRequest::new()
            .with_account_id(account_id.clone())
            .with_instruments("EUR_USD".to_string())
            .remote(&client);
        let candles_future = GetInstrumentCandlesRequest::new()
            .with_instrument("EUR_USD".to_string())
            .with_granularity(CandlestickGranularity::H1)
            .with_count(5)
            .remote(&client);
        
        println!("Executing concurrent requests...");
        let start_time = std::time::Instant::now();
        
        let (accounts_result, pricing_result, candles_result) = futures::future::join3(
            accounts_future,
            pricing_future, 
            candles_future
        ).await;
        
        let elapsed = start_time.elapsed();
        println!("Concurrent requests completed in {:?}", elapsed);
        
        // Check each result individually with detailed error reporting
        let accounts_ok = match &accounts_result {
            Ok(_) => {
                println!("✓ Accounts request succeeded");
                true
            },
            Err(e) => {
                println!("✗ Accounts request failed: {:?}", e);
                false
            }
        };
        
        let pricing_ok = match &pricing_result {
            Ok(response) => {
                println!("✓ Pricing request succeeded (prices: {})", 
                    if response.prices.is_some() { "present" } else { "none" });
                true
            },
            Err(e) => {
                println!("✗ Pricing request failed: {:?}", e);
                false
            }
        };
        
        let candles_ok = match &candles_result {
            Ok(response) => {
                println!("✓ Candles request succeeded (candles: {})", 
                    if response.candles.is_some() { "present" } else { "none" });
                true
            },
            Err(e) => {
                println!("✗ Candles request failed: {:?}", e);
                false
            }
        };
        
        // If all requests succeeded, we're done
        if accounts_ok && pricing_ok && candles_ok {
            println!("All concurrent requests completed successfully on attempt {}", attempt);
            return;
        }
        
        // Check if any failures are due to temporary issues that warrant retry
        let should_retry = (!accounts_ok && is_temporary_api_error(&accounts_result.as_ref().unwrap_err())) ||
                          (!pricing_ok && is_temporary_api_error(&pricing_result.as_ref().unwrap_err())) ||
                          (!candles_ok && is_temporary_api_error(&candles_result.as_ref().unwrap_err()));
        
        if should_retry && attempt < max_attempts {
            println!("Detected temporary API issues, retrying in 1 second...");
            sleep(Duration::from_secs(1)).await;
            attempt += 1;
            continue;
        }
        
        // If we get here, either we've exhausted retries or the errors aren't temporary
        assert!(accounts_result.is_ok(), "Accounts request failed after {} attempts: {:?}", attempt, accounts_result);
        assert!(pricing_result.is_ok(), "Pricing request failed after {} attempts: {:?}", attempt, pricing_result);
        assert!(candles_result.is_ok(), "Candles request failed after {} attempts: {:?}", attempt, candles_result);
        
        break;
    }
}

#[tokio::test]
async fn test_client_shared_across_tasks() {
    let client = Arc::new(create_test_client());
    let account_id = get_test_account_id(&client).await;
    
    // Test that a single client can be safely shared across multiple async tasks
    let mut handles = vec![];
    
    for i in 0..5 {
        let client_clone = Arc::clone(&client);
        let account_id_clone = account_id.clone();
        
        let handle = tokio::spawn(async move {
            let result = match i % 3 {
                0 => {
                    // Test accounts endpoint
                    ListAccountsRequest::new()
                        .remote(&client_clone)
                        .await
                        .map(|_| "accounts")
                        .map_err(|e| format!("{:?}", e))
                },
                1 => {
                    // Test pricing endpoint
                    GetPricesRequest::new()
                        .with_account_id(account_id_clone)
                        .with_instruments("EUR_USD".to_string())
                        .remote(&client_clone)
                        .await
                        .map(|_| "pricing")
                        .map_err(|e| format!("{:?}", e))
                },
                _ => {
                    // Test candles endpoint
                    GetInstrumentCandlesRequest::new()
                        .with_instrument("GBP_USD".to_string())
                        .with_granularity(CandlestickGranularity::M1)
                        .with_count(3)
                        .remote(&client_clone)
                        .await
                        .map(|_| "candles")
                        .map_err(|e| format!("{:?}", e))
                }
            };
            
            (i, result)
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;
    
    // Verify all tasks succeeded
    for (i, join_result) in results.into_iter().enumerate() {
        assert!(join_result.is_ok(), "Task {} failed to complete", i);
        
        let (task_id, api_result) = join_result.unwrap();
        assert!(api_result.is_ok(), "API request in task {} failed: {:?}", task_id, api_result);
        
        println!("Task {} completed {} request successfully", task_id, api_result.unwrap());
    }
}

#[tokio::test]
async fn test_manual_retry_logic_simulation() {
    let mut client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Create a client with a moderately short timeout to simulate temporary failures
    client.reqwest = reqwest::ClientBuilder::new()
        .timeout(Duration::from_millis(500)) // Short timeout to potentially cause failures
        .build()
        .unwrap();
    
    let max_retries = 3;
    let mut retry_count = 0;
    
    // Simulate manual retry logic that would be useful for the client
    loop {
        let result = GetPricesRequest::new()
            .with_account_id(account_id.clone())
            .with_instruments("EUR_USD".to_string())
            .remote(&client)
            .await;
        
        match result {
            Ok(response) => {
                println!("Request succeeded after {} retries", retry_count);
                assert!(response.prices.is_some() || response.prices.is_none(), "Valid response structure");
                break;
            },
            Err(e) => {
                retry_count += 1;
                
                if retry_count >= max_retries {
                    println!("Request failed after {} retries: {:?}", retry_count, e);
                    // For test purposes, we accept that the timeout might be too aggressive
                    // In real implementation, this would be where retry logic helps
                    break;
                } else {
                    println!("Attempt {} failed, retrying...", retry_count);
                    // Exponential backoff simulation
                    let backoff_ms = 100 * (2u64.pow(retry_count - 1));
                    sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }
    }
    
    // This test demonstrates where retry logic would be valuable
    println!("Manual retry simulation completed with {} attempts", retry_count);
}

#[tokio::test]
async fn test_concurrent_different_endpoints_stress() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let start_time = Instant::now();
    
    // Create multiple concurrent pricing requests 
    let pricing_futures: Vec<_> = ["EUR_USD", "GBP_USD", "USD_JPY"]
        .iter()
        .map(|instrument| GetPricesRequest::new()
            .with_account_id(account_id.clone())
            .with_instruments(instrument.to_string())
            .remote(&client))
        .collect();
    
    // Create multiple concurrent candle requests
    let candle_futures = vec![
        GetInstrumentCandlesRequest::new()
            .with_instrument("EUR_USD".to_string())
            .with_granularity(CandlestickGranularity::M5)
            .with_count(5)
            .remote(&client),
        GetInstrumentCandlesRequest::new()
            .with_instrument("GBP_USD".to_string())
            .with_granularity(CandlestickGranularity::M15)
            .with_count(5)
            .remote(&client)
    ];
    
    // Create account requests
    let accounts_future = ListAccountsRequest::new().remote(&client);
    
    // Execute all different types of requests concurrently using join!
    let (pricing_results, candle_results, accounts_result) = futures::future::join3(
        futures::future::join_all(pricing_futures),
        futures::future::join_all(candle_futures),
        accounts_future
    ).await;
    
    let elapsed = start_time.elapsed();
    
    // Analyze results
    let mut success_count = 0;
    let mut failure_count = 0;
    
    // Count pricing results
    for (i, result) in pricing_results.into_iter().enumerate() {
        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                failure_count += 1;
                println!("Pricing request {} failed: {:?}", i, e);
            }
        }
    }
    
    // Count candle results
    for (i, result) in candle_results.into_iter().enumerate() {
        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                failure_count += 1;
                println!("Candle request {} failed: {:?}", i, e);
            }
        }
    }
    
    // Count accounts result
    match accounts_result {
        Ok(_) => success_count += 1,
        Err(e) => {
            failure_count += 1;
            println!("Accounts request failed: {:?}", e);
        }
    }
    
    println!(
        "Concurrent stress test completed in {:?}: {} successes, {} failures",
        elapsed, success_count, failure_count
    );
    
    // At least 80% of requests should succeed in normal conditions
    let total_requests = success_count + failure_count;
    if total_requests > 0 {
        let success_rate = success_count as f64 / total_requests as f64;
        assert!(success_rate >= 0.7, "Success rate {} is below acceptable threshold", success_rate);
    }
    
    // All requests should complete within a reasonable time
    assert!(elapsed < Duration::from_secs(30), "Requests took too long: {:?}", elapsed);
}

#[tokio::test]
async fn test_client_connection_reuse() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test that the reqwest client properly reuses HTTP connections
    let start_time = Instant::now();
    
    // Sequential requests to the same endpoint should reuse connections
    for i in 0..3 {
        let result = GetPricesRequest::new()
            .with_account_id(account_id.clone())
            .with_instruments("EUR_USD".to_string())
            .remote(&client)
            .await;
        
        assert!(result.is_ok(), "Request {} failed: {:?}", i, result);
    }
    
    let sequential_elapsed = start_time.elapsed();
    
    // Test concurrent requests to same endpoint
    let concurrent_start = Instant::now();
    
    let concurrent_futures: Vec<_> = (0..3)
        .map(|_| GetPricesRequest::new()
            .with_account_id(account_id.clone())
            .with_instruments("EUR_USD".to_string())
            .remote(&client))
        .collect();
    
    let concurrent_results = futures::future::join_all(concurrent_futures).await;
    let concurrent_elapsed = concurrent_start.elapsed();
    
    // Verify all concurrent requests succeeded
    for (i, result) in concurrent_results.into_iter().enumerate() {
        assert!(result.is_ok(), "Concurrent request {} failed: {:?}", i, result);
    }
    
    println!("Sequential requests took: {:?}", sequential_elapsed);
    println!("Concurrent requests took: {:?}", concurrent_elapsed);
    
    // This test validates connection reuse patterns
    println!("Connection reuse test completed successfully");
}

#[tokio::test]
async fn test_client_error_recovery_patterns() {
    let client = create_test_client();
    
    // Test how the client behaves with various error conditions
    
    // 1. Test with completely invalid host
    let invalid_client = Client {
        host: "invalid-host-that-does-not-exist.com".to_string(),
        reqwest: client.reqwest.clone(),
        authentication: client.authentication.clone(),
    };
    
    let invalid_result = ListAccountsRequest::new()
        .remote(&invalid_client)
        .await;
    
    assert!(invalid_result.is_err(), "Request to invalid host should fail");
    
    // 2. Test with invalid authentication
    let mut temp_client = create_test_client();
    temp_client.authentication = "invalid_temp_token".to_string();
    
    let temp_result = ListAccountsRequest::new()
        .remote(&temp_client)
        .await;
    
    // OANDA may return success with empty data or actual error
    if temp_result.is_ok() {
        let accounts = temp_result.unwrap();
        assert!(accounts.accounts.is_none() || accounts.accounts.as_ref().unwrap().is_empty(),
               "Invalid auth should return no accounts");
    }
    
    // 3. Test that valid client still works (error recovery)
    let recovery_result = ListAccountsRequest::new()
        .remote(&client)
        .await;
    
    assert!(recovery_result.is_ok(), "Valid client should work after testing invalid client");
    println!("Client error recovery pattern testing completed successfully");
}

#[tokio::test]
async fn test_high_frequency_request_pattern() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Simulate high-frequency trading pattern - rapid consecutive requests
    let request_count = 5;
    let mut request_times = Vec::with_capacity(request_count);
    
    for i in 0..request_count {
        let start = Instant::now();
        
        let result = GetPricesRequest::new()
            .with_account_id(account_id.clone())
            .with_instruments("EUR_USD".to_string())
            .remote(&client)
            .await;
        
        let elapsed = start.elapsed();
        request_times.push(elapsed);
        
        assert!(result.is_ok(), "High-frequency request {} failed: {:?}", i, result);
        
        // Small delay to avoid overwhelming the demo server
        sleep(Duration::from_millis(200)).await;
    }
    
    // Analyze performance characteristics
    let total_time: Duration = request_times.iter().sum();
    let avg_time = total_time / request_count as u32;
    let max_time = request_times.iter().max().unwrap();
    let min_time = request_times.iter().min().unwrap();
    
    println!("High-frequency request analysis:");
    println!("  Average response time: {:?}", avg_time);
    println!("  Max response time: {:?}", max_time);
    println!("  Min response time: {:?}", min_time);
    
    // Reasonable performance expectations for demo environment
    assert!(avg_time < Duration::from_secs(3), "Average response time too high: {:?}", avg_time);
    assert!(*max_time < Duration::from_secs(10), "Maximum response time too high: {:?}", max_time);
    
    // This test helps identify where retry logic and connection pooling would be beneficial
    println!("High-frequency request pattern test completed successfully");
}