use std::fmt::{self, LowerHex};

#[cfg(feature = "serde")]
use serde::Serialize;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::Error;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum EventCode {
    Standard(StandardEventCode),
    Vendor(u16),
    Reserved(u16),
}

impl FromPrimitive for EventCode {
    fn from_i64(_: i64) -> Option<Self> {
        None
    }

    fn from_u64(n: u64) -> Option<Self> {
        let n = n as u16;

        const MSN_MASK: u16 = 0b1111_0000_0000_0000;
        const RESERVED_MSN: u16 = 0b0100;
        const VENDOR_MSN: u16 = 0b1100;

        if let Some(ec) = StandardEventCode::from_u16(n) {
            return Some(EventCode::Standard(ec));
        }

        let msn = (n & MSN_MASK) >> 12;

        if msn == VENDOR_MSN {
            return Some(EventCode::Vendor(n));
        }

        if msn == RESERVED_MSN {
            return Some(EventCode::Reserved(n));
        }

        None
    }
}

impl ToPrimitive for EventCode {
    fn to_i64(&self) -> Option<i64> {
        None
    }

    fn to_u64(&self) -> Option<u64> {
        match self {
            EventCode::Standard(ec) => ec.to_u64(),
            EventCode::Reserved(n) | EventCode::Vendor(n) => Some(*n as u64),
        }
    }
}

impl LowerHex for EventCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventCode::Standard(code) => fmt::LowerHex::fmt(code, f),
            EventCode::Reserved(code) | EventCode::Vendor(code) => fmt::LowerHex::fmt(code, f),
        }
    }
}

impl From<StandardEventCode> for EventCode {
    fn from(code: StandardEventCode) -> Self {
        EventCode::Standard(code)
    }
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum StandardEventCode {
    Undefined = 0x4000,
    CancelTransaction,
    ObjectAdded,
    ObjectRemoved,
    StoreAdded,
    StoreRemoved,
    DevicePropChanged,
    ObjectInfoChanged,
    StoreFull,
    DeviceReset,
    StorageInfoChanged,
    CaptureComplete,
    UnreportedStatus,
}

impl LowerHex for StandardEventCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.to_u16().unwrap();
        fmt::LowerHex::fmt(&val, f)
    }
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Event {
    pub code: EventCode,
    pub params: Vec<u32>,
}

impl Event {
    pub fn new(code: u16, params: &[u8]) -> Result<Self, Error> {
        Ok(Event {
            code: EventCode::from_u16(code).ok_or(Error::BadEventCode)?,
            params: params
                .chunks_exact(4)
                .map(|c| {
                    // TODO: simplify this code when feature(array_chunks) stabilizes
                    // https://github.com/rust-lang/rust/issues/74985
                    let mut b = [0u8; 4];
                    b.copy_from_slice(c);
                    u32::from_be_bytes(b)
                })
                .collect(),
        })
    }
}
