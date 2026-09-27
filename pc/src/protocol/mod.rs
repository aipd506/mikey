mod payloads;
#[cfg(test)]
mod tests;

pub use payloads::{ByePayload, HelloPayload, RejectPayload, WelcomePayload};

use std::io::{self, Error, ErrorKind, Read, Write};

pub const PROTO_VERSION: u32 = 1;
pub const PORT_TCP: u16 = 7653;
pub const PORT_BEACON: u16 = 7654;
pub const PORT_UDP_BEACON: u16 = 7654;
pub const MAX_PAYLOAD_LEN: usize = 4 * 1024 * 1024; // 4 MiB
pub const MEDIA_HEADER_LEN: usize = 14;

pub const CODEC_PCM: u8 = 0x01;
pub const CODEC_OPUS: u8 = 0x02;
pub const CODEC_JPEG: u8 = 0x10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Hello = 0x00,
    Welcome = 0x10,
    Pending = 0x11,
    Reject = 0x12,
    Audio = 0x01,
    Video = 0x02,
    Heartbeat = 0x03,
    Control = 0x04,
    Bye = 0x05,
}

impl FrameType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::Hello),
            0x10 => Some(Self::Welcome),
            0x11 => Some(Self::Pending),
            0x12 => Some(Self::Reject),
            0x01 => Some(Self::Audio),
            0x02 => Some(Self::Video),
            0x03 => Some(Self::Heartbeat),
            0x04 => Some(Self::Control),
            0x05 => Some(Self::Bye),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub frame_type: FrameType,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(frame_type: FrameType, payload: Vec<u8>) -> Self {
        Self {
            frame_type,
            payload,
        }
    }

    pub fn empty(frame_type: FrameType) -> Self {
        Self {
            frame_type,
            payload: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaHeader {
    pub seq: u32,
    pub capture_ts: u64,
    pub codec: u8,
    pub reserved: u8,
}

impl MediaHeader {
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < MEDIA_HEADER_LEN {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "media header truncated",
            ));
        }

        let seq = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let capture_ts = u64::from_be_bytes(bytes[4..12].try_into().unwrap());
        let codec = bytes[12];
        let reserved = bytes[13];

        Ok(Self {
            seq,
            capture_ts,
            codec,
            reserved,
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(MEDIA_HEADER_LEN);
        buf.extend_from_slice(&self.seq.to_be_bytes());
        buf.extend_from_slice(&self.capture_ts.to_be_bytes());
        buf.push(self.codec);
        buf.push(self.reserved);
        buf
    }
}

pub fn read_frame<R: Read>(reader: &mut R) -> io::Result<Frame> {
    loop {
        let mut header = [0u8; 5];
        reader.read_exact(&mut header)?;

        let len = u32::from_be_bytes(header[1..5].try_into().unwrap()) as usize;
        if len > MAX_PAYLOAD_LEN {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("frame payload len {} exceeds max 4 MiB", len),
            ));
        }

        let frame_type = match FrameType::from_u8(header[0]) {
            Some(ft) => ft,
            None => {
                let mut to_skip = len;
                let mut discard_buf = [0u8; 4096];
                while to_skip > 0 {
                    let chunk = to_skip.min(discard_buf.len());
                    reader.read_exact(&mut discard_buf[..chunk])?;
                    to_skip -= chunk;
                }
                continue;
            }
        };

        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload)?;

        return Ok(Frame {
            frame_type,
            payload,
        });
    }
}

pub fn write_frame<W: Write>(writer: &mut W, frame: &Frame) -> io::Result<()> {
    if frame.payload.len() > MAX_PAYLOAD_LEN {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("frame payload {} exceeds max 4 MiB", frame.payload.len()),
        ));
    }

    let mut header = [0u8; 5];
    header[0] = frame.frame_type.to_u8();
    header[1..5].copy_from_slice(&(frame.payload.len() as u32).to_be_bytes());

    writer.write_all(&header)?;
    if !frame.payload.is_empty() {
        writer.write_all(&frame.payload)?;
    }
    writer.flush()?;
    Ok(())
}
