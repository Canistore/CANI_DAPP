use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ErrorCode {
    OperationNotAllowed,
    SystemClose,
    InsufficientFunds,
    InsufficientCycles,
    NonceTooLow,
    NonceTooHigh,
    NoDataFound,
    FailedEncodeArgs,
    FailedSerializeData,
    MaximumRecords,
    DataIsDeleted,
    DataCreateError,
    DataUpdateError,
    DataDeleteError,
    DataIsExists,
    DataNoAssociated,
    DataInvalid,
    ParamsError,
    FailedUpdateState,
    FetchDataError,
    BalanceRetrieveError,
    BalanceTransferError,
    RemoteCallCreateError,
    RemoteCallUpdateError,
    RemoteCallDeleteError,
    RemoteCallBatchCreateError,
    FailedAddTrackToChannel,
    StateNotSetting,
    Other,
}

impl ErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::OperationNotAllowed => "Operation not allowed",
            ErrorCode::SystemClose => "The system is closed",
            ErrorCode::InsufficientFunds => "Transaction failed: Insufficient funds",
            ErrorCode::NonceTooLow => "Transaction failed: Nonce too low",
            ErrorCode::NonceTooHigh => "Transaction failed: Nonce too high",
            ErrorCode::InsufficientCycles => "Insufficient cycles required for deployment",
            ErrorCode::NoDataFound => "No data available",
            ErrorCode::FailedEncodeArgs => "Failed to encode Canister arguments",
            ErrorCode::FailedSerializeData => "Failed to serialize data",
            ErrorCode::MaximumRecords => "The number of records has reached the maximum value",
            ErrorCode::DataIsDeleted => "The data has been deleted",
            ErrorCode::DataCreateError => "Data creation failed",
            ErrorCode::DataUpdateError => "Failed to set Data",
            ErrorCode::DataDeleteError => "Failed to delete Data",
            ErrorCode::DataIsExists => "Data already exists",
            ErrorCode::DataNoAssociated => "Data has no associated",
            ErrorCode::DataInvalid => "Invalid data",
            ErrorCode::ParamsError => "Input parameter error",
            ErrorCode::FetchDataError => "Error fetching data",
            ErrorCode::BalanceRetrieveError => "Failed to retrieve balance",
            ErrorCode::BalanceTransferError => "Failed to execute transfer",
            ErrorCode::RemoteCallCreateError => "Failed to remote create data",
            ErrorCode::RemoteCallUpdateError => "Failed to remote update data",
            ErrorCode::RemoteCallDeleteError => "Failed to remote delete data",
            ErrorCode::RemoteCallBatchCreateError => "Failed to remote batch create data",
            ErrorCode::FailedAddTrackToChannel => "Failed to add track to channel",
            ErrorCode::StateNotSetting => "State Data is not set",
            ErrorCode::FailedUpdateState => "Failed to update state",
            ErrorCode::Other => "Unknown error",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            ErrorCode::OperationNotAllowed => "0001",
            ErrorCode::SystemClose => "0002",
            ErrorCode::InsufficientFunds => "0003",
            ErrorCode::NonceTooLow => "0004",
            ErrorCode::NonceTooHigh => "0005",
            ErrorCode::InsufficientCycles => "0006",
            ErrorCode::NoDataFound => "007",
            ErrorCode::FailedEncodeArgs => "008",
            ErrorCode::FailedSerializeData => "0009",
            ErrorCode::MaximumRecords => "0010",
            ErrorCode::DataIsDeleted => "0011",
            ErrorCode::DataCreateError => "0012",
            ErrorCode::DataUpdateError => "0013",
            ErrorCode::DataDeleteError => "0014",
            ErrorCode::DataIsExists => "0015",
            ErrorCode::DataNoAssociated => "0016",
            ErrorCode::ParamsError => "0017",
            ErrorCode::FetchDataError => "0018",
            ErrorCode::BalanceRetrieveError => "0019",
            ErrorCode::BalanceTransferError => "0020",
            ErrorCode::DataInvalid => "0021",
            ErrorCode::RemoteCallCreateError => "1001",
            ErrorCode::RemoteCallUpdateError => "1002",
            ErrorCode::RemoteCallDeleteError => "1003",
            ErrorCode::RemoteCallBatchCreateError => "1004",
            ErrorCode::FailedAddTrackToChannel => "4001",
            ErrorCode::StateNotSetting => "9997",
            ErrorCode::FailedUpdateState => "9998",
            ErrorCode::Other => "9999",
        }
    }
}

#[derive(Debug)]
pub struct CustomError {
    message: String,
    code: String,
}

impl CustomError {
    pub fn new(code: ErrorCode, message: Option<&str>) -> Self {
        let default_message = code.message().to_string();

        CustomError {
            message: match message {
                Some(custom_message) => format!("{}: {}", default_message, custom_message),
                None => default_message,
            },
            code: code.code().to_string(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}|{}", self.code, self.message)
    }
}

impl Error for CustomError {}
