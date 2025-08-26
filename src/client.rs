use std::error::Error;

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

macro_rules! client_requests {
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

client_requests!( candles(GetInstrumentCandlesRequest) -> GetInstrumentCandlesResponse,
                  orderbook(GetOrderBookRequest) -> GetOrderBookResponse,
                  positionbook(GetPositionBookRequest) -> GetPositionBookResponse,
                  pricing(GetPricesRequest) -> GetPricesResponse,
                  accounts(ListAccountsRequest) -> ListAccountsResponse,
                  account_summary(GetAccountSummaryRequest) -> GetAccountSummaryResponse);
