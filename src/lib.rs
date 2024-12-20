#[cfg(feature = "serde")]
use serde::Serialize;

use byteorder;
use log::{debug, error, trace, warn};
use num_derive::FromPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};
use rusb;
use thiserror::Error;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use std::slice;
use std::sync::Arc;
use std::time::Duration;
use std::{cmp::min, mem::MaybeUninit};
use std::{io, sync::atomic::Ordering};
use std::{io::Cursor, sync::atomic::AtomicU32};

mod command;
mod data;
mod event;
mod response;
mod storage;

pub use crate::command::*;
pub use crate::data::*;
pub use crate::event::*;
pub use crate::response::*;
pub use crate::storage::*;

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u16)]
pub enum ContainerType {
    Command = 1,
    Data = 2,
    Response = 3,
    Event = 4,
}

/// An error in a PTP command
#[derive(Error, Debug)]
pub enum Error {
    /// PTP Responder returned a status code other than Ok, either a constant in StandardResponseCode or a vendor-defined code
    #[error("the ptp device returned an error code: {0:?}")]
    Response(ResponseCode),

    /// Data received was malformed
    #[error("the data received was malformed: {0}")]
    Malformed(String),

    #[error("the data received was malformed: bad object format")]
    BadObjectFormat,

    #[error("the data received was malformed: bad association code")]
    BadAssociationCode,

    #[error("the data received was malformed: bad event code")]
    BadEventCode,

    #[error("received an event with no payload")]
    NoEventPayload,

    /// Another rusb error
    #[error("a usb error occurred: {0}")]
    Usb(#[from] rusb::Error),

    /// Another IO error
    #[error("an i/o error occurred: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DeviceInfo {
    pub version: u16,
    pub vendor_ex_id: u32,
    pub vendor_ex_version: u16,
    pub vendor_extension_desc: String,
    pub functional_mode: u16,
    pub operations_supported: Vec<u16>,
    pub events_supported: Vec<u16>,
    pub device_properties_supported: Vec<u16>,
    pub capture_formats: Vec<u16>,
    pub image_formats: Vec<u16>,
    pub manufacturer: String,
    pub model: String,
    pub device_version: String,
    pub serial_number: String,
}

impl DeviceInfo {
    pub fn decode(buf: &[u8]) -> Result<DeviceInfo, Error> {
        let mut cur = Cursor::new(buf);

        Ok(DeviceInfo {
            version: cur.read_ptp_u16()?,
            vendor_ex_id: cur.read_ptp_u32()?,
            vendor_ex_version: cur.read_ptp_u16()?,
            vendor_extension_desc: cur.read_ptp_str()?,
            functional_mode: cur.read_ptp_u16()?,
            operations_supported: cur.read_ptp_u16_vec()?,
            events_supported: cur.read_ptp_u16_vec()?,
            device_properties_supported: cur.read_ptp_u16_vec()?,
            capture_formats: cur.read_ptp_u16_vec()?,
            image_formats: cur.read_ptp_u16_vec()?,
            manufacturer: cur.read_ptp_str()?,
            model: cur.read_ptp_str()?,
            device_version: cur.read_ptp_str()?,
            serial_number: cur.read_ptp_str()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ObjectInfo {
    pub storage_id: u32,
    pub object_format: ObjectFormatCode,
    pub protection_status: u16,
    pub object_compressed_size: u32,
    pub thumb_format: ObjectFormatCode,
    pub thumb_compressed_size: u32,
    pub thumb_pix_width: u32,
    pub thumb_pix_height: u32,
    pub image_pix_width: u32,
    pub image_pix_height: u32,
    pub image_bit_depth: u32,
    pub parent_object: u32,
    pub association_type: AssociationCode,
    pub association_desc: u32,
    pub sequence_number: u32,
    pub filename: String,
    pub capture_date: String,
    pub modification_date: String,
    pub keywords: String,
}

impl ObjectInfo {
    pub fn decode(buf: &[u8]) -> Result<ObjectInfo, Error> {
        let mut cur = Cursor::new(buf);

        Ok(ObjectInfo {
            storage_id: cur.read_ptp_u32()?,
            object_format: ObjectFormatCode::from_u16(cur.read_ptp_u16()?)
                .ok_or(Error::BadObjectFormat)?,
            protection_status: cur.read_ptp_u16()?,
            object_compressed_size: cur.read_ptp_u32()?,
            thumb_format: ObjectFormatCode::from_u16(cur.read_ptp_u16()?)
                .ok_or(Error::BadObjectFormat)?,
            thumb_compressed_size: cur.read_ptp_u32()?,
            thumb_pix_width: cur.read_ptp_u32()?,
            thumb_pix_height: cur.read_ptp_u32()?,
            image_pix_width: cur.read_ptp_u32()?,
            image_pix_height: cur.read_ptp_u32()?,
            image_bit_depth: cur.read_ptp_u32()?,
            parent_object: cur.read_ptp_u32()?,
            association_type: AssociationCode::from_u16(cur.read_ptp_u16()?)
                .ok_or(Error::BadAssociationCode)?,
            association_desc: cur.read_ptp_u32()?,
            sequence_number: cur.read_ptp_u32()?,
            filename: cur.read_ptp_str()?,
            capture_date: cur.read_ptp_str()?,
            modification_date: cur.read_ptp_str()?,
            keywords: cur.read_ptp_str()?,
        })
    }

    pub fn encode<W: WriteBytesExt>(&self, mut w: W) -> Result<(), Error> {
        w.write_ptp_u32(self.storage_id)?;
        w.write_ptp_u16(self.object_format.to_u16().unwrap())?;
        w.write_ptp_u16(self.protection_status)?;
        w.write_ptp_u32(self.object_compressed_size)?;
        w.write_ptp_u16(self.thumb_format.to_u16().unwrap())?;
        w.write_ptp_u32(self.thumb_compressed_size)?;
        w.write_ptp_u32(self.thumb_pix_width)?;
        w.write_ptp_u32(self.thumb_pix_height)?;
        w.write_ptp_u32(self.image_pix_width)?;
        w.write_ptp_u32(self.image_pix_height)?;
        w.write_ptp_u32(self.image_bit_depth)?;
        w.write_ptp_u32(self.parent_object)?;
        w.write_ptp_u16(self.association_type.to_u16().unwrap())?;
        w.write_ptp_u32(self.association_desc)?;
        w.write_ptp_u32(self.sequence_number)?;
        w.write_ptp_str(&self.filename)?;
        w.write_ptp_str(&self.capture_date)?;
        w.write_ptp_str(&self.modification_date)?;
        w.write_ptp_str(&self.keywords)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct StorageInfo {
    pub storage_type: StorageType,
    pub filesystem_type: FilesystemType,
    pub access_capability: AccessType,
    pub max_capacity: u64,
    pub free_space_in_bytes: u64,
    pub free_space_in_images: u32,
    pub storage_description: String,
    pub volume_label: String,
}

impl StorageInfo {
    pub fn decode<T: PtpRead>(cur: &mut T) -> Result<StorageInfo, Error> {
        Ok(StorageInfo {
            storage_type: StorageType::from_u16(cur.read_ptp_u16()?).unwrap(),
            filesystem_type: FilesystemType::from_u16(cur.read_ptp_u16()?).unwrap(),
            access_capability: AccessType::from_u16(cur.read_ptp_u16()?).unwrap(),
            max_capacity: cur.read_ptp_u64()?,
            free_space_in_bytes: cur.read_ptp_u64()?,
            free_space_in_images: cur.read_ptp_u32()?,
            storage_description: cur.read_ptp_str()?,
            volume_label: cur.read_ptp_str()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum FormData {
    None,
    Range {
        min_value: Data,
        max_value: Data,
        step: Data,
    },
    Enumeration {
        array: Vec<Data>,
    },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PropInfo {
    pub property_code: u16,
    pub data_type: u16,
    pub get_set: u8,
    pub is_enable: u8,
    pub factory_default: Data,
    pub current: Data,
    pub form: FormData,
}

impl PropInfo {
    pub fn decode<T: PtpRead>(cur: &mut T) -> Result<PropInfo, Error> {
        let data_type;
        Ok(PropInfo {
            property_code: cur.read_u16::<LittleEndian>()?,
            data_type: {
                data_type = cur.read_u16::<LittleEndian>()?;
                data_type
            },
            get_set: cur.read_u8()?,
            is_enable: cur.read_u8()?,
            factory_default: Data::read_type(data_type, cur)?,
            current: Data::read_type(data_type, cur)?,
            form: {
                match cur.read_u8()? {
                    // 0x00 => PtpFormData::None,
                    0x01 => FormData::Range {
                        min_value: Data::read_type(data_type, cur)?,
                        max_value: Data::read_type(data_type, cur)?,
                        step: Data::read_type(data_type, cur)?,
                    },
                    0x02 => FormData::Enumeration {
                        array: {
                            let len = cur.read_u16::<LittleEndian>()? as usize;
                            let mut arr = Vec::with_capacity(len);
                            for _ in 0..len {
                                arr.push(Data::read_type(data_type, cur)?);
                            }
                            arr
                        },
                    },
                    _ => FormData::None,
                }
            },
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct ContainerInfo {
    /// payload len in bytes, usually relevant for data phases
    payload_len: usize,

    /// Container kind
    kind: ContainerType,

    /// StandardCommandCode or ResponseCode, depending on 'kind'
    code: u16,

    /// transaction ID that this container belongs to
    tid: u32,
}

const PTP_CONTAINER_INFO_SIZE: usize = 12;

impl ContainerInfo {
    pub fn parse<R: ReadBytesExt>(mut r: R) -> Result<ContainerInfo, Error> {
        let len = r.read_u32::<LittleEndian>()?;
        let kind_u16 = r.read_u16::<LittleEndian>()?;
        let kind = ContainerType::from_u16(kind_u16)
            .ok_or_else(|| Error::Malformed(format!("Invalid message type {:x}.", kind_u16)))?;
        let code = r.read_u16::<LittleEndian>()?;
        let tid = r.read_u32::<LittleEndian>()?;

        Ok(ContainerInfo {
            payload_len: len as usize - PTP_CONTAINER_INFO_SIZE,
            kind,
            code,
            tid,
        })
    }

    // does this container belong to the given transaction?
    pub fn belongs_to(&self, tid: u32) -> bool {
        self.tid == tid
    }
}

pub struct Device<C: rusb::UsbContext> {
    iface: u8,
    ep_in: u8,
    ep_out: u8,
    ep_int: u8,
    current_tid: std::sync::atomic::AtomicU32,
    handle: Arc<rusb::DeviceHandle<C>>,
}

impl<C: rusb::UsbContext> Device<C> {
    pub fn new(handle: Arc<rusb::DeviceHandle<C>>) -> Result<Device<C>, Error> {
        let config_desc = handle.device().active_config_descriptor()?;

        let interface_desc = config_desc
            .interfaces()
            .flat_map(|i| i.descriptors())
            .find(|x| x.class_code() == 6)
            .ok_or(rusb::Error::NotFound)?;

        debug!("Found interface {}", interface_desc.interface_number());

        handle.claim_interface(interface_desc.interface_number())?;
        handle.set_alternate_setting(
            interface_desc.interface_number(),
            interface_desc.setting_number(),
        )?;

        let find_endpoint = |direction, transfer_type| {
            interface_desc
                .endpoint_descriptors()
                .find(|ep| ep.direction() == direction && ep.transfer_type() == transfer_type)
                .map(|x| x.address())
                .ok_or(rusb::Error::NotFound)
        };

        Ok(Device {
            iface: interface_desc.interface_number(),
            ep_in: find_endpoint(rusb::Direction::In, rusb::TransferType::Bulk)?,
            ep_out: find_endpoint(rusb::Direction::Out, rusb::TransferType::Bulk)?,
            ep_int: find_endpoint(rusb::Direction::In, rusb::TransferType::Interrupt)?,
            current_tid: AtomicU32::new(0),
            handle: handle,
        })
    }

    /// Queries the PTP camera for an event. Returns Ok(None) if the operation
    /// times out without receiving an event.
    pub fn event(&self, timeout: Option<Duration>) -> Result<Option<Event>, Error> {
        // timeout of 0 means unlimited timeout.
        let timeout = timeout.unwrap_or(Duration::new(0, 0));

        // let tid = self.current_tid;
        // self.current_tid += 1;

        // read both, check the status on the response, and return the data payload, if any.
        loop {
            let (container, payload) = match self.read_txn_phase_interrupt(timeout) {
                Ok(v) => v,
                Err(Error::Usb(rusb::Error::Timeout)) => return Ok(None),
                Err(e) => return Err(e),
            };

            trace!(
                "event tid: {}, current tid {}",
                container.tid,
                self.current_tid.load(Ordering::Relaxed)
            );

            // if !container.belongs_to(tid) {
            //     return Err(Error::Malformed(format!(
            //         "mismatched txnid {}, expecting {}",
            //         container.tid, tid
            //     )));
            // }

            match container.kind {
                ContainerType::Event => {
                    return Event::new(container.code, payload.as_ref()).map(|p| Some(p));
                }
                _ => {}
            }
        }
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        self.handle.reset()?;

        Ok(())
    }

    /// execute a PTP transaction.
    /// consists of the following phases:
    ///  - command
    ///  - command data (optional, if `data` is Some)
    ///  - response data (optional, if response contains a payload)
    ///  - response status
    /// NB: each phase involves a separate USB transfer, and `timeout` is used for each phase,
    /// so the total time taken may be greater than `timeout`.
    pub fn command(
        &self,
        code: CommandCode,
        params: &[u32],
        data: Option<&[u8]>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>, Error> {
        // timeout of 0 means unlimited timeout.
        let timeout = timeout.unwrap_or(Duration::new(0, 0));

        let tid = self.current_tid.fetch_add(1, Ordering::AcqRel);

        // Prepare payload of the request phase, containing the parameters
        let mut request_payload = Vec::with_capacity(params.len() * 4);
        for p in params {
            request_payload.write_u32::<LittleEndian>(*p).ok();
        }

        self.write_txn_phase(ContainerType::Command, code, tid, &request_payload, timeout)?;

        if let Some(data) = data {
            self.write_txn_phase(ContainerType::Data, code, tid, data, timeout)?;
        }

        // request phase is followed by data phase (optional) and response phase.
        // read both, check the status on the response, and return the data payload, if any.
        let mut data_phase_payload = vec![];
        loop {
            let (container, payload) = self.read_txn_phase_bulk(timeout)?;

            if !container.belongs_to(tid) {
                return Err(Error::Malformed(format!(
                    "mismatched txnid {}, expecting {}",
                    container.tid, tid
                )));
            }

            match container.kind {
                ContainerType::Data => {
                    data_phase_payload = payload;
                }
                ContainerType::Response => {
                    let code = ResponseCode::from_u16(container.code).unwrap();
                    if code != ResponseCode::Standard(StandardResponseCode::Ok) {
                        return Err(Error::Response(code));
                    }
                    return Ok(data_phase_payload);
                }
                _ => {}
            }
        }
    }

    fn write_txn_phase(
        &self,
        kind: ContainerType,
        code: CommandCode,
        tid: u32,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<(), Error> {
        trace!("Write {:?} - 0x{1:04x} ({1:?}), tid:{2}", kind, code, tid);

        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB, must be a multiple of the endpoint packet size

        // The first chunk contains the header, and its payload must be copied into the temporary buffer
        let first_chunk_payload_bytes = min(payload.len(), CHUNK_SIZE - PTP_CONTAINER_INFO_SIZE);
        let mut buf = Vec::with_capacity(first_chunk_payload_bytes + PTP_CONTAINER_INFO_SIZE);
        buf.write_u32::<LittleEndian>((payload.len() + PTP_CONTAINER_INFO_SIZE) as u32)
            .ok();
        buf.write_u16::<LittleEndian>(kind as u16).ok();
        buf.write_u16::<LittleEndian>(code.to_u16().unwrap()).ok();
        buf.write_u32::<LittleEndian>(tid).ok();
        buf.extend_from_slice(&payload[..first_chunk_payload_bytes]);
        self.handle.write_bulk(self.ep_out, &buf, timeout)?;

        // Write any subsequent chunks, straight from the source slice
        for chunk in payload[first_chunk_payload_bytes..].chunks(CHUNK_SIZE) {
            self.handle.write_bulk(self.ep_out, chunk, timeout)?;
        }

        Ok(())
    }

    fn read_txn_phase_bulk(&self, timeout: Duration) -> Result<(ContainerInfo, Vec<u8>), Error> {
        // buf is stack allocated and intended to be large enough to accomodate
        // most cmd/ctrl data (ie, not media) without allocating. payload
        // handling below deals with larger media responses. mark it as
        // uninitalized to avoid paying for zeroing out 8k of memory, since rust
        // doesn't know what rusb does with this memory.

        const BUF_SIZE: usize = 8192;

        let mut buf: MaybeUninit<[u8; BUF_SIZE]> = MaybeUninit::uninit();
        let n =
            self.handle
                .read_bulk(self.ep_in, unsafe { &mut (*buf.as_mut_ptr())[..] }, timeout)?;
        let buf = unsafe { buf.assume_init() };
        let buf = &buf[..n];

        let cinfo = ContainerInfo::parse(&buf[..PTP_CONTAINER_INFO_SIZE])?;
        trace!("container {:?}", cinfo);

        // no payload? we're done
        if cinfo.payload_len == 0 {
            return Ok((cinfo, vec![]));
        }

        // allocate one extra to avoid a separate read for trailing short packet
        let mut payload = Vec::with_capacity(cinfo.payload_len + 1);
        payload.extend_from_slice(&buf[PTP_CONTAINER_INFO_SIZE..]);

        // response didn't fit into our original buf? read the rest
        // or if our original read were satisfied exactly, so there is still a ZLP to read
        if payload.len() < cinfo.payload_len || buf.len() == BUF_SIZE {
            // read in 1MB blocks
            loop {
                unsafe {
                    let p = payload.as_mut_ptr().offset(payload.len() as isize);
                    let pslice = slice::from_raw_parts_mut(
                        p,
                        min(payload.capacity() - payload.len(), 1048576),
                    );
                    let n = self.handle.read_bulk(self.ep_in, pslice, timeout)?;
                    let sz = payload.len();
                    payload.set_len(sz + n);
                    trace!(
                        "  bulk rx {}, ({}/{})",
                        n,
                        payload.len(),
                        payload.capacity()
                    );

                    if n < pslice.len() {
                        break;
                    }
                }
            }
        }

        Ok((cinfo, payload))
    }

    fn read_txn_phase_interrupt(
        &self,
        timeout: Duration,
    ) -> Result<(ContainerInfo, Vec<u8>), Error> {
        let mut buf: [u8; 24] = [0u8; 24];
        let buf = {
            let n = self
                .handle
                .read_interrupt(self.ep_int, &mut buf[..], timeout)?;
            &buf[..n]
        };

        let cinfo = ContainerInfo::parse(&buf[..])?;
        trace!("container {:?}", cinfo);

        // no payload? we're done
        if cinfo.payload_len == 0 {
            warn!("received interrupt data with no payload");

            return Err(Error::NoEventPayload);
        }

        // allocate one extra to avoid a separate read for trailing short packet
        let mut payload = Vec::with_capacity(cinfo.payload_len + 1);
        payload.extend_from_slice(&buf[PTP_CONTAINER_INFO_SIZE..]);

        Ok((cinfo, payload))
    }

    pub fn get_object_info(
        &self,
        handle: ObjectHandle,
        timeout: Option<Duration>,
    ) -> Result<ObjectInfo, Error> {
        let data = self.command(
            StandardCommandCode::GetObjectInfo.into(),
            &[handle.0],
            None,
            timeout,
        )?;
        Ok(ObjectInfo::decode(&data)?)
    }

    pub fn send_object_info(
        &self,
        handle: StorageId,
        parent: ObjectHandle,
        info: ObjectInfo,
        timeout: Option<Duration>,
    ) -> Result<ObjectHandle, Error> {
        let mut data = vec![];
        info.encode(&mut data)?;

        let data = self.command(
            StandardCommandCode::SendObjectInfo.into(),
            &[handle.0, parent.0],
            Some(&data[..]),
            timeout,
        )?;

        let mut cur = Cursor::new(data);
        // storage ID
        let _ = cur.read_ptp_u32()?;
        // parent object handle
        let _ = cur.read_ptp_u32()?;
        let object_handle = cur.read_ptp_u32()?;
        cur.expect_end()?;

        Ok(ObjectHandle(object_handle))
    }

    pub fn send_object(&self, data: &[u8], timeout: Option<Duration>) -> Result<(), Error> {
        self.command(
            StandardCommandCode::SendObject.into(),
            &[],
            Some(data),
            timeout,
        )?;

        Ok(())
    }

    pub fn get_object(
        &self,
        handle: ObjectHandle,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>, Error> {
        self.command(
            StandardCommandCode::GetObject.into(),
            &[handle.0],
            None,
            timeout,
        )
    }

    pub fn get_partial_object(
        &self,
        handle: ObjectHandle,
        offset: u32,
        len: u32,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>, Error> {
        self.command(
            StandardCommandCode::GetPartialObject.into(),
            &[handle.0, offset, len],
            None,
            timeout,
        )
    }

    /// Gets the object handles contained in a storage. If `parent` is not
    /// specified, it will return every object in the storage. If `parent` is
    /// ObjectHandle::root(), then it will return only those at the "root"
    /// level.
    pub fn get_object_handles(
        &self,
        storage_id: StorageId,
        format: Option<ObjectFormatCode>,
        parent: Option<ObjectHandle>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ObjectHandle>, Error> {
        let data = self.command(
            StandardCommandCode::GetObjectHandles.into(),
            &[
                storage_id.0,
                format.map_or(0x0, |fmt| fmt.to_u32().unwrap()),
                parent.map_or(0x0, |p| p.0),
            ],
            None,
            timeout,
        )?;
        // Parse ObjectHandleArrray
        let mut cur = Cursor::new(data);
        let value = cur.read_ptp_u32_vec()?;
        cur.expect_end()?;

        Ok(value.into_iter().map(|oh| ObjectHandle(oh)).collect())
    }

    // handle_id: None == root of store
    pub fn get_num_objects(
        &self,
        storage_id: Option<StorageId>,
        format: Option<ObjectFormatCode>,
        parent: Option<ObjectHandle>,
        timeout: Option<Duration>,
    ) -> Result<u32, Error> {
        let data = self.command(
            StandardCommandCode::GetNumObjects.into(),
            &[
                storage_id.map_or(0xFFFFFFFF, |sid| sid.0),
                format.map_or(0x0, |fmt| fmt.to_u32().unwrap()),
                parent.map_or(0x0, |oh| oh.0),
            ],
            None,
            timeout,
        )?;

        // Parse ObjectHandleArrray
        let mut cur = Cursor::new(data);
        let value = cur.read_ptp_u32()?;
        cur.expect_end()?;

        Ok(value)
    }

    pub fn get_storage_info(
        &self,
        storage_id: StorageId,
        timeout: Option<Duration>,
    ) -> Result<StorageInfo, Error> {
        let data = self.command(
            StandardCommandCode::GetStorageInfo.into(),
            &[storage_id.0],
            None,
            timeout,
        )?;

        // Parse ObjectHandleArrray
        let mut cur = Cursor::new(data);
        let res = StorageInfo::decode(&mut cur)?;
        cur.expect_end()?;

        Ok(res)
    }

    pub fn get_storage_ids(&self, timeout: Option<Duration>) -> Result<Vec<StorageId>, Error> {
        let data = self.command(
            StandardCommandCode::GetStorageIDs.into(),
            &[],
            None,
            timeout,
        )?;

        // Parse ObjectHandleArrray
        let mut cur = Cursor::new(data);
        let value = cur.read_ptp_u32_vec()?;
        cur.expect_end()?;

        Ok(value.into_iter().map(|sid| StorageId(sid)).collect())
    }

    pub fn get_device_info(&self, timeout: Option<Duration>) -> Result<DeviceInfo, Error> {
        let data = self.command(
            StandardCommandCode::GetDeviceInfo.into(),
            &[0, 0, 0],
            None,
            timeout,
        )?;

        let device_info = DeviceInfo::decode(&data)?;
        debug!("device_info {:?}", device_info);
        Ok(device_info)
    }

    pub fn open_session(&self, timeout: Option<Duration>) -> Result<(), Error> {
        let session_id = 3;

        self.command(
            StandardCommandCode::OpenSession.into(),
            &vec![session_id, 0, 0],
            None,
            timeout,
        )?;

        Ok(())
    }

    pub fn close_session(&self, timeout: Option<Duration>) -> Result<(), Error> {
        self.command(StandardCommandCode::CloseSession.into(), &[], None, timeout)?;

        Ok(())
    }

    pub fn disconnect(&mut self, timeout: Option<Duration>) -> Result<(), Error> {
        self.close_session(timeout)?;
        self.handle.release_interface(self.iface)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ObjectTree {
    pub handle: ObjectHandle,
    pub info: ObjectInfo,
    pub children: Option<Vec<ObjectTree>>,
}

impl ObjectTree {
    pub fn walk(&self) -> Vec<(String, ObjectTree)> {
        let mut input = vec![("".to_owned(), self.clone())];
        let mut output = vec![];

        while !input.is_empty() {
            for (prefix, item) in input.split_off(0) {
                let path = prefix.clone()
                    + (if prefix.is_empty() { "" } else { "/" })
                    + &item.info.filename;

                output.push((path.clone(), item.clone()));

                if let Some(children) = item.children {
                    input.extend(children.into_iter().map(|x| (path.clone(), x)));
                }
            }
        }

        output
    }
}
