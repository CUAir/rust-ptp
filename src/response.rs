use std::fmt::{self, LowerHex};
use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum ResponseCode {
    Standard(StandardResponseCode),
    Other(u16),
}

impl FromPrimitive for ResponseCode {
    fn from_i64(n: i64) -> Option<Self> {
        Some(StandardResponseCode::from_i64(n).map_or_else(
            || ResponseCode::Other(n as u16),
            |code| ResponseCode::Standard(code),
        ))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(StandardResponseCode::from_u64(n).map_or_else(
            || ResponseCode::Other(n as u16),
            |code| ResponseCode::Standard(code),
        ))
    }
}

impl LowerHex for ResponseCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseCode::Standard(code) => fmt::LowerHex::fmt(code, f),
            ResponseCode::Other(code) => fmt::LowerHex::fmt(code, f),
        }
    }
}

impl From<StandardResponseCode> for ResponseCode {
    fn from(code: StandardResponseCode) -> Self {
        ResponseCode::Standard(code)
    }
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum StandardResponseCode {
    Undefined = 0x2000,
    Ok = 0x2001,
    GeneralError = 0x2002,
    SessionNotOpen = 0x2003,
    InvalidTransactionId = 0x2004,
    OperationNotSupported = 0x2005,
    ParameterNotSupported = 0x2006,
    IncompleteTransfer = 0x2007,
    InvalidStorageId = 0x2008,
    InvalidObjectHandle = 0x2009,
    DevicePropNotSupported = 0x200A,
    InvalidObjectFormatCode = 0x200B,
    StoreFull = 0x200C,
    ObjectWriteProtected = 0x200D,
    StoreReadOnly = 0x200E,
    AccessDenied = 0x200F,
    NoThumbnailPresent = 0x2010,
    SelfTestFailed = 0x2011,
    PartialDeletion = 0x2012,
    StoreNotAvailable = 0x2013,
    SpecificationByFormatUnsupported = 0x2014,
    NoValidObjectInfo = 0x2015,
    InvalidCodeFormat = 0x2016,
    UnknownVendorCode = 0x2017,
    CaptureAlreadyTerminated = 0x2018,
    DeviceBusy = 0x2019,
    InvalidParentObject = 0x201A,
    InvalidDevicePropFormat = 0x201B,
    InvalidDevicePropValue = 0x201C,
    InvalidParameter = 0x201D,
    SessionAlreadyOpen = 0x201E,
    TransactionCancelled = 0x201F,
    SpecificationOfDestinationUnsupported = 0x2020,
}

impl LowerHex for StandardResponseCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.to_u16().unwrap();
        fmt::LowerHex::fmt(&val, f)
    }
}
