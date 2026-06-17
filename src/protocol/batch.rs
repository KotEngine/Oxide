use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use std::io::{Read, Write};
use crate::utils::Binary;
use crate::protocol::packet::DataPacket;

pub const NETWORK_ID: u8 = 0xfe;
const MAX_BATCH_PACKETS: usize = 500;

pub fn encode_payload_from_packets(packets: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = Vec::new();
    for buf in packets {
        payload.extend(Binary::write_unsigned_var_int(buf.len() as u32));
        payload.extend_from_slice(buf);
    }
    compress(&payload)
}

pub fn decode_packets_from_payload(compressed: &[u8]) -> anyhow::Result<Vec<Vec<u8>>> {
    let payload = decompress(compressed)?;
    let mut offset = 0usize;
    let mut out = Vec::new();
    let mut count = 0usize;
    while offset < payload.len() {
        if count >= MAX_BATCH_PACKETS {
            anyhow::bail!("too many packets in batch");
        }
        let len = Binary::read_unsigned_var_int(&payload, &mut offset)? as usize;
        if offset + len > payload.len() {
            anyhow::bail!("truncated packet in batch");
        }
        out.push(payload[offset..offset + len].to_vec());
        offset += len;
        count += 1;
    }
    Ok(out)
}

fn compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(7));
    encoder.write_all(data).expect("zlib encode");
    encoder.finish().expect("zlib finish")
}

fn decompress(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

pub struct BatchPacket;

impl BatchPacket {
    pub fn encode(packets: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(NETWORK_ID);
        out.extend(encode_payload_from_packets(packets));
        out
    }

    pub fn decode(buf: &[u8]) -> anyhow::Result<Vec<Vec<u8>>> {
        if buf.is_empty() || buf[0] != NETWORK_ID {
            anyhow::bail!("not a batch packet");
        }
        decode_packets_from_payload(&buf[1..])
    }
}
