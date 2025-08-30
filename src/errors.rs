use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RequestValidationError {
    MissingAccountId,
    MissingTradeSpecifier, 
    MissingInstrument,
    MissingTransactionId,
    MissingOrderSpecifier,
    // Add other missing parameter types as needed
}

impl fmt::Display for RequestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestValidationError::MissingAccountId => 
                write!(f, "Account ID is required but was not provided"),
            RequestValidationError::MissingTradeSpecifier => 
                write!(f, "Trade specifier is required but was not provided"),
            RequestValidationError::MissingInstrument => 
                write!(f, "Instrument is required but was not provided"),
            RequestValidationError::MissingTransactionId => 
                write!(f, "Transaction ID is required but was not provided"),
            RequestValidationError::MissingOrderSpecifier => 
                write!(f, "Order specifier is required but was not provided"),
        }
    }
}

impl std::error::Error for RequestValidationError {}

#[derive(Debug, Clone)]
pub enum FxError {
    OrderRejection {
        instrument: String,
        units: String,
        reject_reason: String,
        error_code: String,
        error_message: String,
    },
    ApiError {
        status_code: u16,
        error_code: String,
        error_message: String,
    },
    DeserializationError {
        path: String,
        message: String,
    },
    HttpError(String),
    Validation(RequestValidationError),
}

impl fmt::Display for FxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FxError::OrderRejection { instrument, units, reject_reason, error_code, error_message } => {
                write!(f, "OANDA rejected market order for {} {} units. Reason: {} ({}). Details: {}", 
                       instrument, units, reject_reason, error_code, error_message)
            },
            FxError::ApiError { status_code, error_code, error_message } => {
                write!(f, "OANDA API error (HTTP {}): {} ({})", 
                       status_code, error_message, error_code)
            },
            FxError::DeserializationError { path, message } => {
                write!(f, "Deserialization failed at path '{}': {}", path, message)
            },
            FxError::HttpError(msg) => {
                write!(f, "HTTP request failed: {}", msg)
            },
            FxError::Validation(validation_error) => {
                write!(f, "{}", validation_error)
            },
        }
    }
}

impl std::error::Error for FxError {}

impl From<RequestValidationError> for FxError {
    fn from(err: RequestValidationError) -> Self {
        FxError::Validation(err)
    }
}

impl From<reqwest::Error> for FxError {
    fn from(err: reqwest::Error) -> Self {
        FxError::HttpError(err.to_string())
    }
}

impl From<serde_path_to_error::Error<serde_json::Error>> for FxError {
    fn from(err: serde_path_to_error::Error<serde_json::Error>) -> Self {
        FxError::DeserializationError {
            path: err.path().to_string(),
            message: err.inner().to_string(),
        }
    }
}

// Removed problematic Into implementation

impl From<serde_json::Error> for FxError {
    fn from(err: serde_json::Error) -> Self {
        FxError::DeserializationError {
            path: "unknown".to_string(),
            message: err.to_string(),
        }
    }
}