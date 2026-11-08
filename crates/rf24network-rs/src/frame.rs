use core::fmt::{Display, Formatter};

#[cfg(target_os = "linux")]
pub const MAX_PAYLOAD_SIZE: u16 = 1514;
#[cfg(not(target_os = "linux"))]
pub const MAX_PAYLOAD_SIZE: u8 = 144;

pub struct RF24NetworkHeader {
    pub from_node: u16,
    pub to_node: u16,
    pub frame_id: u16,
    pub message_type: u8,
    pub reserved: u8,
}

impl RF24NetworkHeader {
    pub fn from_buffer(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }
        let mut tmp = [0; 2];
        tmp.copy_from_slice(&buf[0..2]);
        let from_node = u16::from_be_bytes(tmp);
        tmp.copy_from_slice(&buf[2..4]);
        let to_node = u16::from_be_bytes(tmp);
        tmp.copy_from_slice(&buf[4..6]);
        let frame_id = u16::from_be_bytes(tmp);
        let message_type = buf[6];
        let reserved = buf[7];
        Some(Self {
            from_node,
            to_node,
            frame_id,
            message_type,
            reserved,
        })
    }
}

impl Display for RF24NetworkHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "id {} from {:#04o} to {:#04o} type {}",
            self.frame_id, self.from_node, self.to_node, self.message_type
        )
    }
}

pub struct RF24NetworkFrame {
    pub header: RF24NetworkHeader,
    pub message: [u8; 28],
}

impl RF24NetworkFrame {
    pub fn from_buffer(buf: &[u8]) -> Option<Self> {
        let header = RF24NetworkHeader::from_buffer(&buf[0..8]);
        if let Some(header) = header {
            let buf_len = buf.len();
            let mut message = [0; 28];
            message[0..buf_len - 8].copy_from_slice(&buf[8..buf_len]);
            return Some(Self { header, message });
        }
        None
    }
}
