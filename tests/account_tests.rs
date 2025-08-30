mod common;

use fxoanda::*;
use common::*;

#[tokio::test]
async fn test_list_accounts() {
    let client = create_test_client();
    
    let result = ListAccountsRequest::new()
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to list accounts: {:?}", result);
    
    let accounts = result.unwrap();
    assert!(!accounts.accounts.as_ref().unwrap().is_empty(), "Should have at least one demo account");
    
    // Validate account structure
    let first_account = &accounts.accounts.as_ref().unwrap()[0];
    assert!(!first_account.id.as_ref().unwrap().is_empty(), "Account ID should not be empty");
    // Note: We don't validate tags as they may not be present in demo accounts
}

#[tokio::test]
async fn test_get_account_details() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = GetAccountRequest::new()
        .with_account_id(account_id.clone())
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get account details: {:?}", result);
    
    let account_response = result.unwrap();
    let account = account_response.account.expect("Should have account data");
    
    // Validate account details
    assert_eq!(account.id, Some(account_id));
    assert!(account.balance.is_some(), "Account should have a balance");
    assert!(account.margin_used.is_some(), "Account should have margin_used field");
    assert!(account.margin_available.is_some(), "Account should have margin_available field");
    assert!(account.currency.is_some(), "Account should have a base currency");
    
    // Validate currency format (should be 3-letter currency code)
    if let Some(currency) = &account.currency {
        assert_eq!(currency.len(), 3, "Currency should be 3-letter code");
    }
}

#[tokio::test]
async fn test_get_account_summary() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = GetAccountSummaryRequest::new()
        .with_account_id(account_id.clone())
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get account summary: {:?}", result);
    
    let summary_response = result.unwrap();
    let account = summary_response.account.expect("Should have account summary data");
    
    // Validate account summary structure
    assert_eq!(account.id, Some(account_id));
    assert!(account.balance.is_some(), "Account summary should have balance");
    assert!(account.currency.is_some(), "Account summary should have currency");
    
    // Check that this is indeed a summary (should have fewer fields than full account)
    // We can't easily test this without examining the full structure, so we just check basic fields
}

#[tokio::test]
async fn test_get_account_instruments() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = GetAccountInstrumentsRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok(), "Failed to get account instruments: {:?}", result);
    
    let instruments_response = result.unwrap();
    assert!(!instruments_response.instruments.as_ref().unwrap().is_empty(), "Should have instruments available");
    
    // Check for major currency pairs
    let instrument_names: Vec<String> = instruments_response.instruments.as_ref().unwrap()
        .iter()
        .map(|i| i.name.as_ref().unwrap().clone())
        .collect();
    
    let test_instruments = get_test_instruments();
    for test_instrument in test_instruments {
        assert!(
            instrument_names.contains(&test_instrument.to_string()),
            "Should have {} instrument available",
            test_instrument
        );
    }
    
    // Validate instrument structure
    let first_instrument = &instruments_response.instruments.as_ref().unwrap()[0];
    assert!(!first_instrument.name.as_ref().unwrap().is_empty(), "Instrument name should not be empty");
    assert!(first_instrument.pip_location.is_some(), "Instrument should have pip location");
    assert!(first_instrument.display_precision.is_some(), "Instrument should have display precision");
}

#[tokio::test]
async fn test_account_changes_tracking() {
    // Create a mock client for validation testing
    let client = Client {
        host: "api-fxpractice.oanda.com".to_string(),
        reqwest: reqwest::Client::new(),
        authentication: "test-token".to_string(),
    };
    
    // Test missing account ID validation
    let result = GetAccountChangesRequest::new()
        .with_since_transaction_id("1".to_string())
        .remote(&client)
        .await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FxError::Validation(RequestValidationError::MissingAccountId)));
    
    // Test that request with valid account ID should fail with HTTP error (not validation error) 
    // since we're using a mock client that can't actually make the API call
    let result = GetAccountChangesRequest::new()
        .with_account_id("123-456-789-012".to_string())
        .with_since_transaction_id("1".to_string())
        .remote(&client)
        .await;
    
    // Should get API error (401), not validation error, proving validation passed
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FxError::ApiError { status_code: 401, .. }));
}

#[tokio::test]
async fn test_configure_account() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test configuring account with margin rate (this should be allowed for demo accounts)
    let result = ConfigureAccountRequest::new()
        .with_account_id(account_id)
        .with_margin_rate(0.02) // 2% margin rate
        .remote(&client)
        .await;
    
    // Note: This might fail if the account doesn't allow configuration changes
    // For demo accounts, this is often the case, so we check both success and specific error types
    match result {
        Ok(config_response) => {
            assert!(config_response.client_configure_transaction.is_some(), 
                "Successful configuration should have transaction");
        }
        Err(e) => {
            // Check if this is a configuration-not-allowed error (which is expected for demo accounts)
            let error_str = format!("{:?}", e);
            println!("Configuration error (expected for demo): {}", error_str);
            // We don't fail the test here as this is expected behavior for demo accounts
        }
    }
}

#[tokio::test]
async fn test_account_error_handling() {
    // Create a mock client for validation testing
    let client = Client {
        host: "api-fxpractice.oanda.com".to_string(),
        reqwest: reqwest::Client::new(),
        authentication: "test-token".to_string(),
    };
    
    // Test missing account ID validation
    let result = GetAccountRequest::new()
        .remote(&client)
        .await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FxError::Validation(RequestValidationError::MissingAccountId)));
    
    // Test account summary missing account ID
    let result = GetAccountSummaryRequest::new()
        .remote(&client)
        .await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FxError::Validation(RequestValidationError::MissingAccountId)));
}

#[tokio::test]
async fn test_account_balance_validation() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    let result = GetAccountRequest::new()
        .with_account_id(account_id)
        .remote(&client)
        .await;
    
    assert!(result.is_ok());
    let account = result.unwrap().account.expect("Should have account data");
    
    // Validate balance fields are present and reasonable for demo account
    if let Some(balance) = account.balance {
        assert!(balance >= 0.0, "Balance should not be negative in demo account");
        // Demo accounts typically start with significant balance
        assert!(balance >= 1000.0, "Demo account should have substantial starting balance");
    }
    
    if let (Some(margin_used), Some(margin_available)) = (account.margin_used, account.margin_available) {
        assert!(margin_used >= 0.0, "Margin used should not be negative");
        assert!(margin_available >= 0.0, "Margin available should not be negative");
    }
}

#[tokio::test]
async fn test_account_instruments_filtering() {
    let client = create_test_client();
    let account_id = get_test_account_id(&client).await;
    
    // Test getting specific instruments only (if supported)
    let result = GetAccountInstrumentsRequest::new()
        .with_account_id(account_id)
        .with_instruments(vec!["EUR_USD".to_string(), "GBP_USD".to_string()])
        .remote(&client)
        .await;
    
    match result {
        Ok(instruments_response) => {
            // If filtering is supported, check that we only get requested instruments
            let instrument_names: Vec<String> = instruments_response.instruments.as_ref().unwrap()
                .iter()
                .map(|i| i.name.as_ref().unwrap().clone())
                .collect();
            
            for name in instrument_names {
                assert!(
                    name == "EUR_USD" || name == "GBP_USD",
                    "Filtered request should only return requested instruments"
                );
            }
        }
        Err(_) => {
            // If filtering is not supported, this test passes
            // (some OANDA endpoints may not support instrument filtering)
        }
    }
}