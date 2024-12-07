use std::{
    fmt::{LowerHex, UpperHex},
    io::{Cursor, Write},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_traits::ToPrimitive;

use crate::Error;

pub trait PtpRead: ReadBytesExt {
    fn read_ptp_u8(&mut self) -> Result<u8, Error> {
        Ok(self.read_u8()?)
    }

    fn read_ptp_i8(&mut self) -> Result<i8, Error> {
        Ok(self.read_i8()?)
    }

    fn read_ptp_u16(&mut self) -> Result<u16, Error> {
        Ok(self.read_u16::<LittleEndian>()?)
    }

    fn read_ptp_i16(&mut self) -> Result<i16, Error> {
        Ok(self.read_i16::<LittleEndian>()?)
    }

    fn read_ptp_u32(&mut self) -> Result<u32, Error> {
        Ok(self.read_u32::<LittleEndian>()?)
    }

    fn read_ptp_i32(&mut self) -> Result<i32, Error> {
        Ok(self.read_i32::<LittleEndian>()?)
    }

    fn read_ptp_u64(&mut self) -> Result<u64, Error> {
        Ok(self.read_u64::<LittleEndian>()?)
    }

    fn read_ptp_i64(&mut self) -> Result<i64, Error> {
        Ok(self.read_i64::<LittleEndian>()?)
    }

    fn read_ptp_u128(&mut self) -> Result<(u64, u64), Error> {
        let hi = self.read_u64::<LittleEndian>()?;
        let lo = self.read_u64::<LittleEndian>()?;
        Ok((lo, hi))
    }

    fn read_ptp_i128(&mut self) -> Result<(u64, u64), Error> {
        let hi = self.read_u64::<LittleEndian>()?;
        let lo = self.read_u64::<LittleEndian>()?;
        Ok((lo, hi))
    }

    #[inline(always)]
    fn read_ptp_vec<T: Sized, U: Fn(&mut Self) -> Result<T, Error>>(
        &mut self,
        func: U,
    ) -> Result<Vec<T>, Error> {
        let len = self.read_u32::<LittleEndian>()? as usize;
        (0..len).map(|_| func(self)).collect()
    }

    fn read_ptp_u8_vec(&mut self) -> Result<Vec<u8>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_u8())
    }

    fn read_ptp_i8_vec(&mut self) -> Result<Vec<i8>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_i8())
    }

    fn read_ptp_u16_vec(&mut self) -> Result<Vec<u16>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_u16())
    }

    fn read_ptp_i16_vec(&mut self) -> Result<Vec<i16>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_i16())
    }

    fn read_ptp_u32_vec(&mut self) -> Result<Vec<u32>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_u32())
    }

    fn read_ptp_i32_vec(&mut self) -> Result<Vec<i32>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_i32())
    }

    fn read_ptp_u64_vec(&mut self) -> Result<Vec<u64>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_u64())
    }

    fn read_ptp_i64_vec(&mut self) -> Result<Vec<i64>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_i64())
    }

    fn read_ptp_u128_vec(&mut self) -> Result<Vec<(u64, u64)>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_u128())
    }

    fn read_ptp_i128_vec(&mut self) -> Result<Vec<(u64, u64)>, Error> {
        self.read_ptp_vec(|cur| cur.read_ptp_i128())
    }

    fn read_ptp_str(&mut self) -> Result<String, Error> {
        let len = self.read_u8()?;
        if len > 0 {
            // len includes the trailing null u16
            let data: Vec<u16> = (0..(len - 1))
                .map(|_| self.read_u16::<LittleEndian>())
                .collect::<Result<Vec<_>, _>>()?;

            self.read_u16::<LittleEndian>()?;
            String::from_utf16(&data)
                .map_err(|_| Error::Malformed(format!("Invalid UTF16 data: {:?}", data)))
        } else {
            Ok("".into())
        }
    }

    fn expect_end(&mut self) -> Result<(), Error>;
}

impl<T: AsRef<[u8]>> PtpRead for Cursor<T> {
    fn expect_end(&mut self) -> Result<(), Error> {
        let len = self.get_ref().as_ref().len();
        if len as u64 != self.position() {
            Err(Error::Malformed(format!(
                "Response {} bytes, expected {} bytes",
                len,
                self.position()
            )))
        } else {
            Ok(())
        }
    }
}

pub trait PtpWrite: WriteBytesExt {
    fn write_ptp_u8(&mut self, val: u8) -> Result<(), Error> {
        Ok(self.write_u8(val)?)
    }

    fn write_ptp_i8(&mut self, val: i8) -> Result<(), Error> {
        Ok(self.write_i8(val)?)
    }

    fn write_ptp_u16(&mut self, val: u16) -> Result<(), Error> {
        Ok(self.write_u16::<LittleEndian>(val)?)
    }

    fn write_ptp_i16(&mut self, val: i16) -> Result<(), Error> {
        Ok(self.write_i16::<LittleEndian>(val)?)
    }

    fn write_ptp_u32(&mut self, val: u32) -> Result<(), Error> {
        Ok(self.write_u32::<LittleEndian>(val)?)
    }

    fn write_ptp_i32(&mut self, val: i32) -> Result<(), Error> {
        Ok(self.write_i32::<LittleEndian>(val)?)
    }

    fn write_ptp_u64(&mut self, val: u64) -> Result<(), Error> {
        Ok(self.write_u64::<LittleEndian>(val)?)
    }

    fn write_ptp_i64(&mut self, val: i64) -> Result<(), Error> {
        Ok(self.write_i64::<LittleEndian>(val)?)
    }

    fn write_ptp_u128(&mut self, val: (u64, u64)) -> Result<(), Error> {
        let (lo, hi) = val;
        self.write_u64::<LittleEndian>(lo)?;
        Ok(self.write_u64::<LittleEndian>(hi)?)
    }

    fn write_ptp_i128(&mut self, val: (u64, u64)) -> Result<(), Error> {
        let (lo, hi) = val;
        self.write_u64::<LittleEndian>(lo)?;
        Ok(self.write_u64::<LittleEndian>(hi)?)
    }

    fn write_ptp_vec<T, F>(&mut self, items: &[T], write_fn: F) -> Result<(), Error>
    where
        F: Fn(&mut Self, &T) -> Result<(), Error>,
    {
        self.write_u32::<LittleEndian>(items.len() as u32)?;
        for item in items {
            write_fn(self, item)?;
        }
        Ok(())
    }

    fn write_ptp_str(&mut self, val: &str) -> Result<(), Error> {
        let utf16: Vec<u16> = val.encode_utf16().collect();
        self.write_u8((utf16.len() as u8) * 2 + 1)?;
        if !utf16.is_empty() {
            for c in utf16 {
                self.write_u16::<LittleEndian>(c)?;
            }
            self.write_all(&[0, 0])?;
        }
        Ok(())
    }
}

impl<W: WriteBytesExt> PtpWrite for W {}

#[derive(Debug, Eq, PartialEq, PartialOrd, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Data {
    UNDEF,
    INT8(i8),
    UINT8(u8),
    INT16(i16),
    UINT16(u16),
    INT32(i32),
    UINT32(u32),
    INT64(i64),
    UINT64(u64),
    INT128((u64, u64)),
    UINT128((u64, u64)),
    AINT8(Vec<i8>),
    AUINT8(Vec<u8>),
    AINT16(Vec<i16>),
    AUINT16(Vec<u16>),
    AINT32(Vec<i32>),
    AUINT32(Vec<u32>),
    AINT64(Vec<i64>),
    AUINT64(Vec<u64>),
    AINT128(Vec<(u64, u64)>),
    AUINT128(Vec<(u64, u64)>),
    STR(String),
}

impl Data {
    pub fn encode(&self) -> Vec<u8> {
        use self::Data::*;
        let mut out = vec![];
        match self {
            // UNDEF => {},
            &INT8(val) => {
                out.write_i8(val).ok();
            }
            &UINT8(val) => {
                out.write_u8(val).ok();
            }
            &INT16(val) => {
                out.write_i16::<LittleEndian>(val).ok();
            }
            &UINT16(val) => {
                out.write_u16::<LittleEndian>(val).ok();
            }
            &INT32(val) => {
                out.write_i32::<LittleEndian>(val).ok();
            }
            &UINT32(val) => {
                out.write_u32::<LittleEndian>(val).ok();
            }
            &INT64(val) => {
                out.write_i64::<LittleEndian>(val).ok();
            }
            &UINT64(val) => {
                out.write_u64::<LittleEndian>(val).ok();
            }
            &INT128((hi, lo)) => {
                out.write_u64::<LittleEndian>(lo).ok();
                out.write_u64::<LittleEndian>(hi).ok();
            }
            &UINT128((hi, lo)) => {
                out.write_u64::<LittleEndian>(lo).ok();
                out.write_u64::<LittleEndian>(hi).ok();
            }
            &AINT8(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_i8(*item).ok();
                }
            }
            &AUINT8(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_u8(*item).ok();
                }
            }
            &AINT16(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_i16::<LittleEndian>(*item).ok();
                }
            }
            &AUINT16(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_u16::<LittleEndian>(*item).ok();
                }
            }
            &AINT32(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_i32::<LittleEndian>(*item).ok();
                }
            }
            &AUINT32(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_u32::<LittleEndian>(*item).ok();
                }
            }
            &AINT64(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_i64::<LittleEndian>(*item).ok();
                }
            }
            &AUINT64(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for item in val {
                    out.write_u64::<LittleEndian>(*item).ok();
                }
            }
            &AINT128(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for &(hi, lo) in val {
                    out.write_u64::<LittleEndian>(lo).ok();
                    out.write_u64::<LittleEndian>(hi).ok();
                }
            }
            &AUINT128(ref val) => {
                out.write_u32::<LittleEndian>(val.len() as u32).ok();
                for &(hi, lo) in val {
                    out.write_u64::<LittleEndian>(lo).ok();
                    out.write_u64::<LittleEndian>(hi).ok();
                }
            }
            &STR(ref val) => {
                out.write_u8(((val.len() as u8) * 2) + 1).ok();
                if val.len() > 0 {
                    for e in val.encode_utf16() {
                        out.write_u16::<LittleEndian>(e).ok();
                    }
                    out.write_all(b"\0\0").ok();
                }
            }
            _ => {}
        }
        out
    }

    pub fn read_type<T: PtpRead>(kind: u16, reader: &mut T) -> Result<Data, Error> {
        use self::Data::*;
        Ok(match kind {
            // 0x0000 => UNDEF,
            0x0001 => INT8(reader.read_ptp_i8()?),
            0x0002 => UINT8(reader.read_ptp_u8()?),
            0x0003 => INT16(reader.read_ptp_i16()?),
            0x0004 => UINT16(reader.read_ptp_u16()?),
            0x0005 => INT32(reader.read_ptp_i32()?),
            0x0006 => UINT32(reader.read_ptp_u32()?),
            0x0007 => INT64(reader.read_ptp_i64()?),
            0x0008 => UINT64(reader.read_ptp_u64()?),
            0x0009 => INT128(reader.read_ptp_i128()?),
            0x000A => UINT128(reader.read_ptp_u128()?),
            0x4001 => AINT8(reader.read_ptp_i8_vec()?),
            0x4002 => AUINT8(reader.read_ptp_u8_vec()?),
            0x4003 => AINT16(reader.read_ptp_i16_vec()?),
            0x4004 => AUINT16(reader.read_ptp_u16_vec()?),
            0x4005 => AINT32(reader.read_ptp_i32_vec()?),
            0x4006 => AUINT32(reader.read_ptp_u32_vec()?),
            0x4007 => AINT64(reader.read_ptp_i64_vec()?),
            0x4008 => AUINT64(reader.read_ptp_u64_vec()?),
            0x4009 => AINT128(reader.read_ptp_i128_vec()?),
            0x400A => AUINT128(reader.read_ptp_u128_vec()?),
            0xFFFF => STR(reader.read_ptp_str()?),
            _ => UNDEF,
        })
    }
}

impl ToPrimitive for Data {
    fn to_i64(&self) -> Option<i64> {
        match self {
            Data::INT8(v) => Some(*v as i64),
            Data::UINT8(v) => Some(*v as i64),
            Data::INT16(v) => Some(*v as i64),
            Data::UINT16(v) => Some(*v as i64),
            Data::INT32(v) => Some(*v as i64),
            Data::UINT32(v) => Some(*v as i64),
            Data::INT64(v) => Some(*v as i64),
            _ => None,
        }
    }

    fn to_u64(&self) -> Option<u64> {
        match self {
            Data::UINT8(v) => Some(*v as u64),
            Data::UINT16(v) => Some(*v as u64),
            Data::UINT32(v) => Some(*v as u64),
            Data::UINT64(v) => Some(*v as u64),
            _ => None,
        }
    }
}

impl LowerHex for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::UNDEF => write!(f, "UNDEF"),
            Data::INT8(v) => write!(f, "INT8({:#04x})", v),
            Data::UINT8(v) => write!(f, "UINT8({:#04x})", v),
            Data::INT16(v) => write!(f, "INT16({:#06x})", v),
            Data::UINT16(v) => write!(f, "UINT16({:#06x})", v),
            Data::INT32(v) => write!(f, "INT32({:#010x})", v),
            Data::UINT32(v) => write!(f, "UINT32({:#010x})", v),
            Data::INT64(v) => write!(f, "INT64({:#018x})", v),
            Data::UINT64(v) => write!(f, "UINT64({:#018x})", v),
            Data::INT128((h, l)) => {
                write!(f, "INT128({:#034x})", ((*h as i128) << 64) | (*l as i128))
            }
            Data::UINT128((h, l)) => {
                write!(f, "UINT128({:#034x})", ((*h as u128) << 64) | (*l as u128))
            }
            Data::AINT8(v) => {
                write!(f, "AINT8(")?;
                for i in &v[..] {
                    write!(f, "{:02x}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT8(v) => {
                write!(f, "AUINT8(")?;
                for i in &v[..] {
                    write!(f, "{:02x}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT16(v) => {
                write!(f, "AINT16(")?;
                for i in &v[..] {
                    write!(f, "{:04x}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT16(v) => {
                write!(f, "AUINT16(")?;
                for i in &v[..] {
                    write!(f, "{:04x}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT32(v) => {
                write!(f, "AINT32(")?;
                for i in &v[..] {
                    write!(f, "{:08x}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT32(v) => {
                write!(f, "AUINT32(")?;
                for i in &v[..] {
                    write!(f, "{:08x}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT64(v) => {
                write!(f, "AINT64(")?;
                for i in &v[..] {
                    write!(f, "{:016x}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT64(v) => {
                write!(f, "AUINT64(")?;
                for i in &v[..] {
                    write!(f, "{:016x}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT128(v) => {
                write!(f, "AINT128(")?;
                for (h, l) in &v[..] {
                    write!(f, "{:032x}", ((*h as i128) << 64) | (*l as i128))?;
                }
                write!(f, ")")
            }
            Data::AUINT128(v) => {
                write!(f, "AUINT128(")?;
                for (h, l) in &v[..] {
                    write!(f, "{:032x}", ((*h as u128) << 64) | (*l as u128))?;
                }
                write!(f, ")")
            }
            Data::STR(v) => write!(f, "STR({:?})", v),
        }
    }
}

impl UpperHex for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::UNDEF => write!(f, "UNDEF"),
            Data::INT8(v) => write!(f, "INT8({:#04x})", v),
            Data::UINT8(v) => write!(f, "UINT8({:#04x})", v),
            Data::INT16(v) => write!(f, "INT16({:#06x})", v),
            Data::UINT16(v) => write!(f, "UINT16({:#06x})", v),
            Data::INT32(v) => write!(f, "INT32({:#010x})", v),
            Data::UINT32(v) => write!(f, "UINT32({:#010x})", v),
            Data::INT64(v) => write!(f, "INT64({:#018x})", v),
            Data::UINT64(v) => write!(f, "UINT64({:#018x})", v),
            Data::INT128((h, l)) => {
                write!(f, "INT128({:#034x})", ((*h as i128) << 64) | (*l as i128))
            }
            Data::UINT128((h, l)) => {
                write!(f, "UINT128({:#034x})", ((*h as u128) << 64) | (*l as u128))
            }
            Data::AINT8(v) => {
                write!(f, "AINT8(")?;
                for i in &v[..] {
                    write!(f, "{:02X}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT8(v) => {
                write!(f, "AUINT8(")?;
                for i in &v[..] {
                    write!(f, "{:02X}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT16(v) => {
                write!(f, "AINT16(")?;
                for i in &v[..] {
                    write!(f, "{:04X}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT16(v) => {
                write!(f, "AUINT16(")?;
                for i in &v[..] {
                    write!(f, "{:04X}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT32(v) => {
                write!(f, "AINT32(")?;
                for i in &v[..] {
                    write!(f, "{:08X}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT32(v) => {
                write!(f, "AUINT32(")?;
                for i in &v[..] {
                    write!(f, "{:08X}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT64(v) => {
                write!(f, "AINT64(")?;
                for i in &v[..] {
                    write!(f, "{:016X}", i)?;
                }
                write!(f, ")")
            }
            Data::AUINT64(v) => {
                write!(f, "AUINT64(")?;
                for i in &v[..] {
                    write!(f, "{:016X}", i)?;
                }
                write!(f, ")")
            }
            Data::AINT128(v) => {
                write!(f, "AINT128(")?;
                for (h, l) in &v[..] {
                    write!(f, "{:032X}", ((*h as i128) << 64) | (*l as i128))?;
                }
                write!(f, ")")
            }
            Data::AUINT128(v) => {
                write!(f, "AUINT128(")?;
                for (h, l) in &v[..] {
                    write!(f, "{:032X}", ((*h as u128) << 64) | (*l as u128))?;
                }
                write!(f, ")")
            }
            Data::STR(v) => write!(f, "STR({:?})", v),
        }
    }
}

impl<'a> From<i8> for Data {
    fn from(value: i8) -> Self {
        Data::INT8(value)
    }
}

impl<'a> From<u8> for Data {
    fn from(value: u8) -> Self {
        Data::UINT8(value)
    }
}

impl<'a> From<i16> for Data {
    fn from(value: i16) -> Self {
        Data::INT16(value)
    }
}

impl<'a> From<u16> for Data {
    fn from(value: u16) -> Self {
        Data::UINT16(value)
    }
}

impl<'a> From<i32> for Data {
    fn from(value: i32) -> Self {
        Data::INT32(value)
    }
}

impl<'a> From<u32> for Data {
    fn from(value: u32) -> Self {
        Data::UINT32(value)
    }
}

impl<'a> From<i64> for Data {
    fn from(value: i64) -> Self {
        Data::INT64(value)
    }
}

impl<'a> From<u64> for Data {
    fn from(value: u64) -> Self {
        Data::UINT64(value)
    }
}

impl<'a> From<&'a str> for Data {
    fn from(value: &'a str) -> Self {
        Data::STR(value.to_owned())
    }
}

impl<'a> From<String> for Data {
    fn from(value: String) -> Self {
        Data::STR(value)
    }
}
