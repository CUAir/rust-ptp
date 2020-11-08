use num_traits::{FromPrimitive, ToPrimitive};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive, Ord, PartialOrd, Eq)]
pub struct ObjectHandle(pub(crate) u32);

impl ObjectHandle {
    /// Returns an ObjectHandle that represents the "root" of a storage.
    pub fn root() -> ObjectHandle {
        ObjectHandle(0xFFFFFFFF)
    }
}

impl Display for ObjectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjectHandle(0x{:08x})", self.0)
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive, Ord, PartialOrd, Eq)]
pub enum StandardObjectFormatCode {
    UndefinedNonImage = 0x3000,
    Association,
    Script,
    Executable,
    Text,
    Html,
    Dpof,
    Aiff,
    Wav,
    Mp3,
    Avi,
    Mpeg,
    Asf,
    UndefinedImage = 0x3800,
    ExifJpeg,
    TiffEp,
    FlashPix,
    Bmp,
    Ciff,
    UndefinedReserved,
    Gif,
    Jfif,
    Pcd,
    Pict,
    Png,
    UndefinedReserved2,
    Tiff,
    TiffIt,
    Jp2,
    Jpx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFormatCode {
    Standard(StandardObjectFormatCode),
    Reserved(u16),
    Vendor(u16),
    ImageOnly,
}

impl FromPrimitive for ObjectFormatCode {
    fn from_i64(_: i64) -> Option<Self> {
        None
    }

    fn from_u64(n: u64) -> Option<Self> {
        let n = n as u16;

        const MSN_MASK: u16 = 0b1111_0000_0000_0000;
        const RESERVED_MSN: u16 = 0b0011;
        const VENDOR_MSN: u16 = 0b1011;

        if let Some(ofc) = StandardObjectFormatCode::from_u16(n) {
            return Some(ObjectFormatCode::Standard(ofc));
        }

        if n == 0xFFFF {
            return Some(ObjectFormatCode::ImageOnly);
        }

        let msn = (n & MSN_MASK) >> 12;

        if msn == VENDOR_MSN {
            return Some(ObjectFormatCode::Vendor(n));
        }

        if msn == RESERVED_MSN {
            return Some(ObjectFormatCode::Reserved(n));
        }

        None
    }
}

impl ToPrimitive for ObjectFormatCode {
    fn to_i64(&self) -> Option<i64> {
        None
    }

    fn to_u64(&self) -> Option<u64> {
        match self {
            ObjectFormatCode::Standard(ofc) => ofc.to_u64(),
            ObjectFormatCode::Reserved(n) | ObjectFormatCode::Vendor(n) => Some(*n as u64),
            ObjectFormatCode::ImageOnly => Some(0xFFFFFFFF),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive, Ord, PartialOrd, Eq)]
pub struct StorageId(pub(crate) u32);

impl StorageId {
    /// Returns a StorageId that represents all of the storages combined.
    pub fn all() -> StorageId {
        StorageId(0xFFFFFFFF)
    }
}

impl Display for StorageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StorageId(0x{:08x})", self.0)
    }
}
