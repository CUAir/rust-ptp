use std::fmt::{self, LowerHex};
use num_traits::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum CommandCode {
    Standard(StandardCommandCode),
    Other(u16),
}

impl LowerHex for CommandCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandCode::Standard(code) => fmt::LowerHex::fmt(code, f),
            CommandCode::Other(code) => fmt::LowerHex::fmt(code, f),
        }
    }
}

impl FromPrimitive for CommandCode {
    fn from_i64(n: i64) -> Option<Self> {
        Some(StandardCommandCode::from_i64(n).map_or_else(
            || CommandCode::Other(n as u16),
            |code| CommandCode::Standard(code),
        ))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(StandardCommandCode::from_u64(n).map_or_else(
            || CommandCode::Other(n as u16),
            |code| CommandCode::Standard(code),
        ))
    }
}

impl ToPrimitive for CommandCode {
    fn to_i64(&self) -> Option<i64> {
        match self {
            CommandCode::Standard(code) => code.to_i64(),
            CommandCode::Other(code) => Some(*code as i64),
        }
    }

    fn to_u64(&self) -> Option<u64> {
        match self {
            CommandCode::Standard(code) => code.to_u64(),
            CommandCode::Other(code) => Some(*code as u64),
        }
    }
}

impl From<StandardCommandCode> for CommandCode {
    fn from(code: StandardCommandCode) -> Self {
        CommandCode::Standard(code)
    }
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum StandardCommandCode {
    Undefined = 0x1000,
    GetDeviceInfo = 0x1001,
    OpenSession = 0x1002,
    CloseSession = 0x1003,
    GetStorageIDs = 0x1004,
    GetStorageInfo = 0x1005,
    GetNumObjects = 0x1006,
    GetObjectHandles = 0x1007,
    GetObjectInfo = 0x1008,
    GetObject = 0x1009,
    GetThumb = 0x100A,
    DeleteObject = 0x100B,
    SendObjectInfo = 0x100C,
    SendObject = 0x100D,
    InitiateCapture = 0x100E,
    FormatStore = 0x100F,
    ResetDevice = 0x1010,
    SelfTest = 0x1011,
    SetObjectProtection = 0x1012,
    PowerDown = 0x1013,
    GetDevicePropDesc = 0x1014,
    GetDevicePropValue = 0x1015,
    SetDevicePropValue = 0x1016,
    ResetDevicePropValue = 0x1017,
    TerminateOpenCapture = 0x1018,
    MoveObject = 0x1019,
    CopyObject = 0x101A,
    GetPartialObject = 0x101B,
    InitiateOpenCapture = 0x101C,
}

impl LowerHex for StandardCommandCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.to_u16().unwrap();
        fmt::LowerHex::fmt(&val, f)
    }
}
