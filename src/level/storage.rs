use std::io::{Read, Write};
use std::path::Path;
use flate2::Compression;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};

use crate::level::chunk::{Chunk, MAX_SUBCHUNKS};
use crate::level::chunk::chunk::{BIOMES_LEN, HEIGHTMAP_LEN};
use crate::level::chunk::sub_chunk::{BLOCK_DATA_LEN, BLOCK_IDS_LEN, BLOCK_LIGHT_LEN, SKY_LIGHT_LEN};
use crate::nbt::{NbtCompound, NbtReader, NbtValue, NbtWriter};
use crate::protocol::handler::PlayerState;
use crate::protocol::types::Item;
use crate::utils::BinaryStream;

pub fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

// --- WORLD LEVEL.DAT PERSISTENCE ---

#[derive(Debug, Clone)]
pub struct LevelDat {
    pub level_name: String,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub time: i32,
    pub seed: i64,
    pub generator_name: String,
}

pub fn save_level_dat(world_dir: &Path, dat: &LevelDat) -> anyhow::Result<()> {
    let mut data_compound = NbtCompound::new();
    data_compound.insert("LevelName".into(), NbtValue::String(dat.level_name.clone()));
    data_compound.insert("SpawnX".into(), NbtValue::Int(dat.spawn_x));
    data_compound.insert("SpawnY".into(), NbtValue::Int(dat.spawn_y));
    data_compound.insert("SpawnZ".into(), NbtValue::Int(dat.spawn_z));
    data_compound.insert("Time".into(), NbtValue::Long(dat.time as i64));
    data_compound.insert("RandomSeed".into(), NbtValue::Long(dat.seed));
    data_compound.insert("generatorName".into(), NbtValue::String(dat.generator_name.clone()));

    let mut root = NbtCompound::new();
    root.insert("Data".into(), NbtValue::Compound(data_compound));

    let mut writer = NbtWriter::new();
    writer.write_root("", &NbtValue::Compound(root));
    let raw = writer.into_bytes();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw)?;
    let compressed = encoder.finish()?;

    atomic_write(&world_dir.join("level.dat"), &compressed)?;
    Ok(())
}

pub fn load_level_dat(world_dir: &Path) -> Option<LevelDat> {
    let path = world_dir.join("level.dat");
    let bytes = std::fs::read(&path).ok()?;
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw).ok()?;

    let mut reader = NbtReader::new(raw);
    let (_, root_val) = reader.read_root().ok()?;
    let root = root_val.as_compound()?;
    let data = root.get("Data").and_then(|v| v.as_compound()).unwrap_or(root);

    let level_name = data
        .get("LevelName")
        .and_then(|v| v.as_string())
        .unwrap_or("world")
        .to_string();
    let spawn_x = data.get("SpawnX").and_then(|v| v.as_int()).unwrap_or(0);
    let spawn_y = data.get("SpawnY").and_then(|v| v.as_int()).unwrap_or(64);
    let spawn_z = data.get("SpawnZ").and_then(|v| v.as_int()).unwrap_or(0);
    let time = data.get("Time").and_then(|v| v.as_int()).unwrap_or(0);
    let seed = match data.get("RandomSeed") {
        Some(NbtValue::Long(l)) => *l,
        Some(other) => other.as_int().unwrap_or(0) as i64,
        None => 0,
    };
    let generator_name = data
        .get("generatorName")
        .and_then(|v| v.as_string())
        .unwrap_or("default")
        .to_string();

    Some(LevelDat {
        level_name,
        spawn_x,
        spawn_y,
        spawn_z,
        time,
        seed,
        generator_name,
    })
}

// --- CHUNK BINARY + ZLIB PERSISTENCE ---

pub fn encode_chunk_binary(chunk: &Chunk) -> Vec<u8> {
    let mut raw = Vec::with_capacity(8192);
    raw.extend_from_slice(b"OXC1");
    for i in 0..BIOMES_LEN {
        raw.push(*chunk.biome_ids.get(i).unwrap_or(&1));
    }
    for i in 0..HEIGHTMAP_LEN {
        let h = *chunk.height_map.get(i).unwrap_or(&256);
        raw.extend_from_slice(&h.to_le_bytes());
    }
    let mut sub_mask = 0u16;
    for (idx, sub) in chunk.sub_chunks.iter().enumerate() {
        if idx < 16 && !sub.is_empty() {
            sub_mask |= 1u16 << idx;
        }
    }
    raw.extend_from_slice(&sub_mask.to_le_bytes());
    for idx in 0..MAX_SUBCHUNKS.min(16) {
        if (sub_mask & (1u16 << idx)) != 0 {
            let sub = &chunk.sub_chunks[idx];
            raw.extend_from_slice(&sub.block_ids);
            raw.extend_from_slice(&sub.block_data);
            raw.extend_from_slice(&sub.block_light);
            raw.extend_from_slice(&sub.sky_light);
        }
    }
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::new(3));
    let _ = enc.write_all(&raw);
    enc.finish().unwrap_or_default()
}

pub fn decode_chunk_binary(cx: i32, cz: i32, compressed: &[u8]) -> Option<Chunk> {
    let mut dec = ZlibDecoder::new(compressed);
    let mut raw = Vec::new();
    dec.read_to_end(&mut raw).ok()?;

    let min_header = 4 + BIOMES_LEN + HEIGHTMAP_LEN * 2 + 2;
    if raw.len() < min_header || &raw[0..4] != b"OXC1" {
        return None;
    }

    let mut pos = 4;
    let mut chunk = Chunk::new(cx, cz);
    chunk.biome_ids = raw[pos..pos + BIOMES_LEN].to_vec();
    pos += BIOMES_LEN;

    for i in 0..HEIGHTMAP_LEN {
        chunk.height_map[i] = u16::from_le_bytes([raw[pos], raw[pos + 1]]);
        pos += 2;
    }

    let sub_mask = u16::from_le_bytes([raw[pos], raw[pos + 1]]);
    pos += 2;

    let sub_stride = BLOCK_IDS_LEN + BLOCK_DATA_LEN + BLOCK_LIGHT_LEN + SKY_LIGHT_LEN;
    for idx in 0..MAX_SUBCHUNKS.min(16) {
        if (sub_mask & (1u16 << idx)) != 0 {
            if pos + sub_stride > raw.len() {
                return None;
            }
            let sub = &mut chunk.sub_chunks[idx];
            sub.block_ids.copy_from_slice(&raw[pos..pos + BLOCK_IDS_LEN]);
            pos += BLOCK_IDS_LEN;
            sub.block_data.copy_from_slice(&raw[pos..pos + BLOCK_DATA_LEN]);
            pos += BLOCK_DATA_LEN;
            sub.block_light.copy_from_slice(&raw[pos..pos + BLOCK_LIGHT_LEN]);
            pos += BLOCK_LIGHT_LEN;
            sub.sky_light.copy_from_slice(&raw[pos..pos + SKY_LIGHT_LEN]);
            pos += SKY_LIGHT_LEN;
        }
    }

    Some(chunk)
}

pub fn chunk_file_path(world_dir: &Path, cx: i32, cz: i32) -> std::path::PathBuf {
    world_dir.join("chunks").join(format!("c.{}.{}.oxc", cx, cz))
}

pub fn save_chunk_to_disk(world_dir: &Path, chunk: &Chunk) -> anyhow::Result<()> {
    let encoded = encode_chunk_binary(chunk);
    atomic_write(&chunk_file_path(world_dir, chunk.x, chunk.z), &encoded)?;
    Ok(())
}

pub fn load_chunk_from_disk(world_dir: &Path, cx: i32, cz: i32) -> Option<Chunk> {
    let bytes = std::fs::read(chunk_file_path(world_dir, cx, cz)).ok()?;
    decode_chunk_binary(cx, cz, &bytes)
}

// --- PLAYER .DAT PERSISTENCE ---

#[derive(Debug, Clone)]
pub struct LoadedPlayerData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub dimension: i32,
    pub gamemode: i32,
    pub health: u32,
    pub food_level: u32,
    pub food_saturation: f32,
    pub exhaustion: f32,
    pub air_ticks: i16,
    pub fire_ticks: i32,
    pub xp_level: u32,
    pub xp_progress: f32,
    pub xp_points: u32,
    pub spawn_pos: Option<(i32, i32, i32, i32)>,
    pub inventory: Vec<Item>,
    pub armor: Vec<Item>,
    pub offhand: Item,
    pub ender_chest: Vec<Item>,
    pub hotbar: [i32; 9],
    pub held_index: usize,
}

fn item_to_nbt(slot: usize, item: &Item) -> NbtValue {
    let mut c = NbtCompound::new();
    c.insert("Slot".into(), NbtValue::Byte(slot as i8));
    c.insert("id".into(), NbtValue::Short(item.id as i16));
    c.insert("Damage".into(), NbtValue::Short(item.meta as i16));
    c.insert("Count".into(), NbtValue::Byte(item.count as i8));
    if !item.nbt.is_empty() {
        let signed: Vec<i8> = item.nbt.iter().map(|&b| b as i8).collect();
        c.insert("RawTag".into(), NbtValue::ByteArray(signed));
    }
    NbtValue::Compound(c)
}

fn nbt_to_item(val: &NbtValue) -> Option<(usize, Item)> {
    let c = val.as_compound()?;
    let slot = c.get("Slot").and_then(|v| v.as_int()).unwrap_or(0) as usize;
    let id = c.get("id").and_then(|v| v.as_int()).unwrap_or(0);
    let meta = c.get("Damage").and_then(|v| v.as_int()).unwrap_or(0);
    let count = c.get("Count").and_then(|v| v.as_int()).unwrap_or(0).max(0) as u32;
    let nbt = match c.get("RawTag") {
        Some(NbtValue::ByteArray(arr)) => arr.iter().map(|&b| b as u8).collect(),
        _ => Vec::new(),
    };
    Some((slot, Item { id, meta, count, nbt }))
}

pub fn save_player_dat(data_dir: &Path, state: &PlayerState) -> anyhow::Result<()> {
    if state.username.is_empty() {
        return Ok(());
    }
    let mut root = NbtCompound::new();
    root.insert(
        "Pos".into(),
        NbtValue::List(vec![
            NbtValue::Double(state.x),
            NbtValue::Double(state.y),
            NbtValue::Double(state.z),
        ]),
    );
    root.insert(
        "Rotation".into(),
        NbtValue::List(vec![NbtValue::Float(state.yaw), NbtValue::Float(state.pitch)]),
    );
    root.insert("Dimension".into(), NbtValue::Int(state.dimension));
    root.insert("playerGameType".into(), NbtValue::Int(state.gamemode));
    root.insert("Health".into(), NbtValue::Short(state.health as i16));
    root.insert("foodLevel".into(), NbtValue::Int(state.food_level as i32));
    root.insert("foodSaturationLevel".into(), NbtValue::Float(state.food_saturation));
    root.insert("foodExhaustionLevel".into(), NbtValue::Float(state.exhaustion));
    root.insert("Air".into(), NbtValue::Short(state.air_ticks));
    root.insert("Fire".into(), NbtValue::Short(state.fire_ticks as i16));
    root.insert("XpLevel".into(), NbtValue::Int(state.xp_level as i32));
    root.insert("XpP".into(), NbtValue::Float(state.xp_progress));
    root.insert("XpTotal".into(), NbtValue::Int(state.xp_points as i32));
    root.insert("SelectedInventorySlot".into(), NbtValue::Int(state.inventory.held_index() as i32));
    root.insert("HotbarLinks".into(), NbtValue::IntArray(state.inventory.hotbar_array().to_vec()));

    if let Some((sx, sy, sz, sdim)) = state.spawn_pos {
        root.insert("SpawnX".into(), NbtValue::Int(sx));
        root.insert("SpawnY".into(), NbtValue::Int(sy));
        root.insert("SpawnZ".into(), NbtValue::Int(sz));
        root.insert("SpawnDim".into(), NbtValue::Int(sdim));
    }

    let inv_list: Vec<NbtValue> = state
        .inventory
        .get_contents()
        .iter()
        .enumerate()
        .filter(|(_, it)| !it.is_air() && it.count > 0)
        .map(|(i, it)| item_to_nbt(i, it))
        .collect();
    root.insert("Inventory".into(), NbtValue::List(inv_list));

    let armor_list: Vec<NbtValue> = state
        .inventory
        .armor_contents()
        .iter()
        .enumerate()
        .filter(|(_, it)| !it.is_air() && it.count > 0)
        .map(|(i, it)| item_to_nbt(i, it))
        .collect();
    root.insert("Armor".into(), NbtValue::List(armor_list));

    let off = state.inventory.offhand();
    if !off.is_air() && off.count > 0 {
        root.insert("Offhand".into(), item_to_nbt(0, &off));
    }

    let ender_list: Vec<NbtValue> = state
        .ender_chest
        .iter()
        .enumerate()
        .filter(|(_, it)| !it.is_air() && it.count > 0)
        .map(|(i, it)| item_to_nbt(i, it))
        .collect();
    root.insert("EnderChestInventory".into(), NbtValue::List(ender_list));

    let mut writer = NbtWriter::new();
    writer.write_root("", &NbtValue::Compound(root));
    let raw = writer.into_bytes();

    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&raw)?;
    let compressed = enc.finish()?;

    let file_path = data_dir
        .join("players")
        .join(format!("{}.dat", state.username.to_lowercase()));
    atomic_write(&file_path, &compressed)?;
    Ok(())
}

pub fn load_player_dat(data_dir: &Path, username: &str) -> Option<LoadedPlayerData> {
    let file_path = data_dir
        .join("players")
        .join(format!("{}.dat", username.to_lowercase()));
    let bytes = std::fs::read(&file_path).ok()?;
    let mut dec = GzDecoder::new(&bytes[..]);
    let mut raw = Vec::new();
    dec.read_to_end(&mut raw).ok()?;

    let mut reader = NbtReader::new(raw);
    let (_, val) = reader.read_root().ok()?;
    let root = val.as_compound()?;

    let get_double = |v: &NbtValue| -> Option<f64> {
        match v {
            NbtValue::Double(d) => Some(*d),
            NbtValue::Float(f) => Some(*f as f64),
            _ => None,
        }
    };
    let get_float = |v: &NbtValue| -> Option<f32> {
        match v {
            NbtValue::Float(f) => Some(*f),
            NbtValue::Double(d) => Some(*d as f32),
            _ => None,
        }
    };

    let (x, y, z) = match root.get("Pos").and_then(|v| v.as_list()) {
        Some(l) if l.len() >= 3 => (
            get_double(&l[0]).unwrap_or(0.0),
            get_double(&l[1]).unwrap_or(64.0),
            get_double(&l[2]).unwrap_or(0.0),
        ),
        _ => return None,
    };
    let (yaw, pitch) = match root.get("Rotation").and_then(|v| v.as_list()) {
        Some(l) if l.len() >= 2 => (
            get_float(&l[0]).unwrap_or(0.0),
            get_float(&l[1]).unwrap_or(0.0),
        ),
        _ => (0.0, 0.0),
    };

    let dimension = root.get("Dimension").and_then(|v| v.as_int()).unwrap_or(0);
    let gamemode = root.get("playerGameType").and_then(|v| v.as_int()).unwrap_or(0);
    let health = root.get("Health").and_then(|v| v.as_int()).unwrap_or(20).clamp(0, 20) as u32;
    let food_level = root.get("foodLevel").and_then(|v| v.as_int()).unwrap_or(20).clamp(0, 20) as u32;
    let food_saturation = root.get("foodSaturationLevel").and_then(get_float).unwrap_or(5.0);
    let exhaustion = root.get("foodExhaustionLevel").and_then(get_float).unwrap_or(0.0);
    let air_ticks = root.get("Air").and_then(|v| v.as_int()).unwrap_or(400) as i16;
    let fire_ticks = root.get("Fire").and_then(|v| v.as_int()).unwrap_or(0).max(0);
    let xp_level = root.get("XpLevel").and_then(|v| v.as_int()).unwrap_or(0).max(0) as u32;
    let xp_progress = root.get("XpP").and_then(get_float).unwrap_or(0.0);
    let xp_points = root.get("XpTotal").and_then(|v| v.as_int()).unwrap_or(0).max(0) as u32;
    let held_index = (root.get("SelectedInventorySlot").and_then(|v| v.as_int()).unwrap_or(0) as usize).min(8);

    let mut hotbar = [0i32, 1, 2, 3, 4, 5, 6, 7, 8];
    if let Some(NbtValue::IntArray(arr)) = root.get("HotbarLinks") {
        for (i, &v) in arr.iter().take(9).enumerate() {
            hotbar[i] = v;
        }
    }

    let spawn_pos = match (
        root.get("SpawnX").and_then(|v| v.as_int()),
        root.get("SpawnY").and_then(|v| v.as_int()),
        root.get("SpawnZ").and_then(|v| v.as_int()),
    ) {
        (Some(sx), Some(sy), Some(sz)) => {
            let sdim = root.get("SpawnDim").and_then(|v| v.as_int()).unwrap_or(0);
            Some((sx, sy, sz, sdim))
        }
        _ => None,
    };

    let mut inventory = vec![Item::air(); 36];
    if let Some(list) = root.get("Inventory").and_then(|v| v.as_list()) {
        for v in list {
            if let Some((slot, it)) = nbt_to_item(v) {
                if slot < inventory.len() {
                    inventory[slot] = it;
                }
            }
        }
    }

    let mut armor = vec![Item::air(); 4];
    if let Some(list) = root.get("Armor").and_then(|v| v.as_list()) {
        for v in list {
            if let Some((slot, it)) = nbt_to_item(v) {
                if slot < armor.len() {
                    armor[slot] = it;
                }
            }
        }
    }

    let offhand = root
        .get("Offhand")
        .and_then(nbt_to_item)
        .map(|(_, it)| it)
        .unwrap_or_else(Item::air);

    let mut ender_chest = vec![Item::air(); 27];
    if let Some(list) = root.get("EnderChestInventory").and_then(|v| v.as_list()) {
        for v in list {
            if let Some((slot, it)) = nbt_to_item(v) {
                if slot < ender_chest.len() {
                    ender_chest[slot] = it;
                }
            }
        }
    }

    Some(LoadedPlayerData {
        x,
        y,
        z,
        yaw,
        pitch,
        dimension,
        gamemode,
        health,
        food_level,
        food_saturation,
        exhaustion,
        air_ticks,
        fire_ticks,
        xp_level,
        xp_progress,
        xp_points,
        spawn_pos,
        inventory,
        armor,
        offhand,
        ender_chest,
        hotbar,
        held_index,
    })
}

pub fn save_bytes_compressed(path: &Path, raw: &[u8]) -> std::io::Result<()> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::new(3));
    enc.write_all(raw)?;
    let out = enc.finish()?;
    atomic_write(path, &out)
}

pub fn load_bytes_compressed(path: &Path) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).ok()?;
    let mut dec = ZlibDecoder::new(&bytes[..]);
    let mut raw = Vec::new();
    dec.read_to_end(&mut raw).ok()?;
    Some(raw)
}

pub fn put_item_binary(s: &mut BinaryStream, item: &Item) {
    s.put_var_int(item.id);
    if !item.is_air() {
        s.put_var_int(item.meta);
        s.put_unsigned_var_int(item.count);
        s.put_unsigned_var_int(item.nbt.len() as u32);
        s.put(&item.nbt);
    }
}

pub fn get_item_binary(s: &mut BinaryStream) -> Item {
    let id = s.get_var_int().unwrap_or(0);
    if id <= 0 {
        return Item::air();
    }
    let meta = s.get_var_int().unwrap_or(0);
    let count = s.get_unsigned_var_int().unwrap_or(1);
    let nbt_len = s.get_unsigned_var_int().unwrap_or(0) as usize;
    let nbt = if nbt_len > 0 {
        s.get(nbt_len).unwrap_or_default()
    } else {
        Vec::new()
    };
    Item { id, meta, count, nbt }
}
