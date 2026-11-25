use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use std::io::{Read, Write};
use crate::utils::Binary;

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
    tracing::debug!("batch decompress: {} compressed bytes", compressed.len());
    let payload = decompress(compressed)?;
    tracing::debug!("batch decompress: {} decompressed bytes", payload.len());
    let mut offset = 0usize;
    let mut out = Vec::new();
    let mut count = 0usize;
    while offset < payload.len() {
        if count >= MAX_BATCH_PACKETS {
            anyhow::bail!("too many packets in batch");
        }
        let len = match Binary::read_unsigned_var_int(&payload, &mut offset) {
            Ok(v) => v as usize,
            Err(e) => {
                tracing::warn!("batch varint read failed at offset {}: {}", offset, e);
                anyhow::bail!("batch varint read failed: {}", e);
            }
        };
        if offset + len > payload.len() {
            tracing::warn!("batch truncated: need {} at offset {}, have {}", len, offset, payload.len());
            anyhow::bail!("truncated packet in batch");
        }
        let pkt = payload[offset..offset + len].to_vec();
        if !pkt.is_empty() {
            tracing::debug!("batch packet {}: id=0x{:02x}, {} bytes", count, pkt[0], pkt.len());
        }
        out.push(pkt);
        offset += len;
        count += 1;
    }
    tracing::debug!("batch decoded {} packets", count);
    Ok(out)
}

fn compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(3));
    encoder.write_all(data).expect("zlib encode");
    encoder.finish().expect("zlib finish")
}

fn decompress(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    match decoder.read_to_end(&mut out) {
        Ok(_) => Ok(out),
        Err(e) => {
            tracing::warn!("batch zlib decompress failed: {} (input {} bytes)", e, data.len());
            Err(anyhow::anyhow!("zlib decompress failed: {}", e))
        }
    }
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
            anyhow::bail!("not a batch packet (first byte = 0x{:02x})", if buf.is_empty() { 0 } else { buf[0] });
        }
        decode_packets_from_payload(&buf[1..])
    }
}
