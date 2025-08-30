use fxoanda::*;

fn create_mock_client() -> Client {
    Client {
        host: "api-fxpractice.oanda.com".to_string(),
        reqwest: reqwest::Client::new(),
        authentication: "test-token".to_string(),
    }
}

#[cfg(test)]
mod test_account_errors {
    use super::*;

    #[tokio::test]
    async fn test_get_trade_missing_account_id() {
        let client = create_mock_client();
        let request = GetTradeRequest::new()
            .with_trade_specifier("123".to_string());
        
        let result = request.remote(&client).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, FxError::Validation(RequestValidationError::MissingAccountId)));
    }

    #[tokio::test] 
    async fn test_get_trade_missing_trade_specifier() {
        let client = create_mock_client();
        let request = GetTradeRequest::new()
            .with_account_id("123-456-789-012".to_string());
        
        let result = request.remote(&client).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, FxError::Validation(RequestValidationError::MissingTradeSpecifier)));
    }

    #[tokio::test]
    async fn test_get_position_missing_instrument() {
        let client = create_mock_client();
        let request = GetPositionRequest::new()
            .with_account_id("123-456-789-012".to_string());
        
        let result = request.remote(&client).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, FxError::Validation(RequestValidationError::MissingInstrument)));
    }

    #[tokio::test]
    async fn test_close_position_missing_multiple_params() {
        let client = create_mock_client();
        let request = ClosePositionRequest::new();
        
        let result = request.remote(&client).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should fail on first missing parameter (account_id)
        assert!(matches!(error, FxError::Validation(RequestValidationError::MissingAccountId)));
    }
}

#[cfg(test)]
mod test_instrument_errors {
    use super::*;

    #[tokio::test]
    async fn test_get_candles_missing_instrument() {
        let client = create_mock_client();
        let request = GetInstrumentCandlesRequest::new();
        
        let result = request.remote(&client).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, FxError::Validation(RequestValidationError::MissingInstrument)));
    }

    #[tokio::test]
    async fn test_get_order_book_missing_instrument() {
        let client = create_mock_client();
        let request = GetOrderBookRequest::new();
        
        let result = request.remote(&client).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, FxError::Validation(RequestValidationError::MissingInstrument)));
    }

    #[tokio::test]
    async fn test_get_position_book_missing_instrument() {
        let client = create_mock_client();
        let request = GetPositionBookRequest::new();
        
        let result = request.remote(&client).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, FxError::Validation(RequestValidationError::MissingInstrument)));
    }
}

#[cfg(test)]
mod test_pricing_errors {
    // Note: We need to identify what pricing requests have path parameters
    // For now, implementing a placeholder test
    #[tokio::test]
    async fn test_pricing_placeholder() {
        // This test will be implemented once we identify the specific
        // pricing requests that have unwrap() calls with path parameters
        assert!(true, "Placeholder for pricing error tests");
    }
}