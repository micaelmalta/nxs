/// Minimal .nxb decoder — reads the preamble and walks the root object,
/// returning a flat list of (key_index, value_bytes) for inspection.
use crate::error::{NxsError, Result};

const MAGIC_FILE: u32 = 0x4E585342;
const MAGIC_OBJ: u32  = 0x4E58534F;
const MAGIC_FOOTER: u32 = 0x2153584E;

pub struct DecodedFile {
    pub version: u16,
    pub flags: u16,
    pub dict_hash: u64,
    pub tail_ptr: u64,
    pub keys: Vec<String>,
    pub root_fields: Vec<(String, DecodedValue)>,
}

#[derive(Debug, Clone)]
pub enum DecodedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Time(i64),
    Binary(Vec<u8>),
    Null,
    Raw(Vec<u8>),
}

pub fn decode(data: &[u8]) -> Result<DecodedFile> {
    if data.len() < 32 {
        return Err(NxsError::OutOfBounds);
    }

    let magic = u32::from_le_bytes(data[0..4].try_into().map_err(|_| NxsError::OutOfBounds)?);
    if magic != MAGIC_FILE {
        return Err(NxsError::BadMagic);
    }

    let footer_magic = u32::from_le_bytes(data[data.len()-4..].try_into().map_err(|_| NxsError::OutOfBounds)?);
    if footer_magic != MAGIC_FOOTER {
        return Err(NxsError::BadMagic);
    }

    let version   = u16::from_le_bytes(data[4..6].try_into().map_err(|_| NxsError::OutOfBounds)?);
    let flags     = u16::from_le_bytes(data[6..8].try_into().map_err(|_| NxsError::OutOfBounds)?);
    let dict_hash = u64::from_le_bytes(data[8..16].try_into().map_err(|_| NxsError::OutOfBounds)?);
    let tail_ptr  = u64::from_le_bytes(data[16..24].try_into().map_err(|_| NxsError::OutOfBounds)?);

    let schema_embedded = flags & 0x0002 != 0;
    let mut pos = 32usize;
    let mut keys: Vec<String> = Vec::new();

    if schema_embedded && pos < data.len() {
        let key_count = u16::from_le_bytes(data[pos..pos+2].try_into().map_err(|_| NxsError::OutOfBounds)?) as usize;
        pos += 2;
        // skip TypeManifest
        pos += key_count;
        // read StringPool
        for _ in 0..key_count {
            let start = pos;
            while pos < data.len() && data[pos] != 0 { pos += 1; }
            let name = String::from_utf8_lossy(&data[start..pos]).to_string();
            keys.push(name);
            pos += 1; // skip null terminator
        }
        // align to 8
        while pos % 8 != 0 { pos += 1; }
    }

    // Decode root object
    let root_fields = decode_object(data, pos, &keys)?;

    Ok(DecodedFile { version, flags, dict_hash, tail_ptr, keys, root_fields })
}

fn decode_object(data: &[u8], offset: usize, keys: &[String]) -> Result<Vec<(String, DecodedValue)>> {
    let mut pos = offset;

    if pos + 8 > data.len() { return Err(NxsError::OutOfBounds); }
    let magic = u32::from_le_bytes(data[pos..pos+4].try_into().map_err(|_| NxsError::OutOfBounds)?);
    if magic != MAGIC_OBJ { return Err(NxsError::BadMagic); }
    pos += 4;

    let obj_len = u32::from_le_bytes(data[pos..pos+4].try_into().map_err(|_| NxsError::OutOfBounds)?) as usize;
    pos += 4;

    // Read LEB128 bitmask
    let mut present_bits: Vec<bool> = Vec::new();
    loop {
        if pos >= data.len() { return Err(NxsError::OutOfBounds); }
        let byte = data[pos]; pos += 1;
        for bit in 0..7 {
            present_bits.push((byte >> bit) & 1 == 1);
        }
        if byte & 0x80 == 0 { break; }
    }

    // Count present fields
    let present_count = present_bits.iter().filter(|&&b| b).count();

    // Read offset table (u16 each, normal mode)
    let mut offsets: Vec<usize> = Vec::new();
    for _ in 0..present_count {
        if pos + 2 > data.len() { return Err(NxsError::OutOfBounds); }
        let off = u16::from_le_bytes(data[pos..pos+2].try_into().map_err(|_| NxsError::OutOfBounds)?) as usize;
        offsets.push(offset + off);
        pos += 2;
    }

    // Map each present bit to its key and decode its value
    let mut fields = Vec::new();
    let mut offset_idx = 0;
    for (bit_idx, &present) in present_bits.iter().enumerate() {
        if !present { continue; }
        let key_name = keys.get(bit_idx)
            .cloned()
            .unwrap_or_else(|| format!("key_{bit_idx}"));
        let val_offset = offsets[offset_idx];
        offset_idx += 1;

        // Determine type from the TypeManifest sigil for this key
        // For simplicity we peek at the raw bytes and try to infer
        let value = decode_value_at(data, val_offset, obj_len)?;
        fields.push((key_name, value));
    }

    Ok(fields)
}

fn decode_value_at(data: &[u8], offset: usize, _obj_len: usize) -> Result<DecodedValue> {
    if offset >= data.len() { return Err(NxsError::OutOfBounds); }

    // Check if it looks like a nested object
    if offset + 4 <= data.len() {
        let maybe_magic = u32::from_le_bytes(data[offset..offset+4].try_into().map_err(|_| NxsError::OutOfBounds)?);
        if maybe_magic == MAGIC_OBJ {
            // Return raw bytes for nested objects (full recursive decode omitted for brevity)
            return Ok(DecodedValue::Raw(data[offset..offset+8.min(data.len()-offset)].to_vec()));
        }
    }

    // Null (single 0x00 byte)
    if offset < data.len() && data[offset] == 0x00 {
        // Ambiguous with zero integer, but works for our POC output
        return Ok(DecodedValue::Null);
    }

    // Try i64 (8 bytes)
    if offset + 8 <= data.len() {
        let raw = &data[offset..offset+8];
        let i = i64::from_le_bytes(raw.try_into().map_err(|_| NxsError::OutOfBounds)?);
        // If the high bytes are padding (could be bool), check length-prefix patterns
        // For POC we return raw i64 and let caller interpret
        return Ok(DecodedValue::Int(i));
    }

    Ok(DecodedValue::Raw(data[offset..].to_vec()))
}
