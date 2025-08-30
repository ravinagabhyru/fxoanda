use crate::*;

/// The main client for interacting with the Oanda V20 API.
///
/// The client holds the HTTP client, host information, and authentication details
/// required to make authenticated requests to the API endpoints.
///
/// # Example
///
/// ```no_run
/// # use std::env;
/// # use fxoanda::Client;
/// let api_key = env::var("OANDA_KEY").unwrap();
/// let api_host = env::var("OANDA_HOST").unwrap();
///
/// let client = Client {
///     host: api_host,
///     reqwest: reqwest::Client::new(),
///     authentication: api_key,
/// };
/// ```
#[derive(Debug)]
pub struct Client {
    /// The `reqwest::Client` used for making HTTP requests.
    pub reqwest: reqwest::Client,
    /// The Oanda API host (e.g., "api-fxpractice.oanda.com"). Do not include `https://`.
    pub host: String,
    /// The Oanda API authentication token (API Key).
    pub authentication: String,
}

// Macro for modules that still return Box<dyn Error>
macro_rules! client_requests_old {
    ($($func:ident($request:ident) -> $response:ident),*) => {
      $(         
         impl Client {
           pub async fn $func( &self, x: $request ) -> Result<$response, Box<dyn Error>> {
             x.remote(&self).await
           }
         }
       )*
    };
}

// Macro for migrated modules that return FxError
macro_rules! client_requests_new {
    ($($func:ident($request:ident) -> $response:ident),*) => {
      $(         
         impl Client {
           pub async fn $func( &self, x: $request ) -> Result<$response, FxError> {
             x.remote(&self).await
           }
         }
       )*
    };
}

client_requests_old!( 
                      // accounts(ListAccountsRequest) -> ListAccountsResponse // TODO: Fix ListAccountsRequest generation
                      );

client_requests_new!( 
    // Account functions
    list_accounts(ListAccountsRequest) -> ListAccountsResponse,
    get_account(GetAccountRequest) -> GetAccountResponse,
    account_summary(GetAccountSummaryRequest) -> GetAccountSummaryResponse,
    account_instruments(GetAccountInstrumentsRequest) -> GetAccountInstrumentsResponse,
    configure_account(ConfigureAccountRequest) -> ConfigureAccountResponse,
    account_changes(GetAccountChangesRequest) -> GetAccountChangesResponse,
    // Position functions
    list_positions(ListPositionsRequest) -> ListPositionsResponse,
    list_open_positions(ListOpenPositionsRequest) -> ListOpenPositionsResponse,
    get_position(GetPositionRequest) -> GetPositionResponse,
    close_position(ClosePositionRequest) -> ClosePositionResponse,
    // Trade functions
    list_trades(ListTradesRequest) -> ListTradesResponse,
    list_open_trades(ListOpenTradesRequest) -> ListOpenTradesResponse,
    get_trade(GetTradeRequest) -> GetTradeResponse,
    close_trade(CloseTradeRequest) -> CloseTradeResponse,
    set_trade_client_extensions(SetTradeClientExtensionsRequest) -> SetTradeClientExtensionsResponse,
    set_trade_dependent_orders(SetTradeDependentOrdersRequest) -> SetTradeDependentOrdersResponse,
    // Order functions
    create_market_order(CreateMarketOrderRequest) -> CreateMarketOrderResponse,
    create_limit_order(CreateLimitOrderRequest) -> CreateLimitOrderResponse,
    create_stop_order(CreateStopOrderRequest) -> CreateStopOrderResponse,
    list_orders(ListOrdersRequest) -> ListOrdersResponse,
    list_pending_orders(ListPendingOrdersRequest) -> ListPendingOrdersResponse,
    get_order(GetOrderRequest) -> GetOrderResponse,
    replace_order(ReplaceOrderRequest) -> ReplaceOrderResponse,
    cancel_order(CancelOrderRequest) -> CancelOrderResponse,
    set_order_client_extensions(SetOrderClientExtensionsRequest) -> SetOrderClientExtensionsResponse,
    // Transaction functions
    list_transactions(ListTransactionsRequest) -> ListTransactionsResponse,
    get_transaction(GetTransactionRequest) -> GetTransactionResponse,
    get_transaction_range(GetTransactionRangeRequest) -> GetTransactionRangeResponse,
    get_transactions_since_id(GetTransactionsSinceIdRequest) -> GetTransactionsSinceIdResponse,
    stream_transactions(StreamTransactionsRequest) -> StreamTransactionsResponse,
    // Pricing functions
    pricing(GetPricesRequest) -> GetPricesResponse,
    stream_pricing(StreamPricingRequest) -> StreamPricingResponse,
    account_instrument_candles(GetAccountInstrumentCandlesRequest) -> GetAccountInstrumentCandlesResponse,
    // Instrument functions  
    candles(GetInstrumentCandlesRequest) -> GetInstrumentCandlesResponse,
    orderbook(GetOrderBookRequest) -> GetOrderBookResponse,
    positionbook(GetPositionBookRequest) -> GetPositionBookResponse
);
