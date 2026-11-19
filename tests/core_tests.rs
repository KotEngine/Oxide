use oxide::block::BlockRegistry;
use oxide::item::ItemRegistry;
use oxide::inventory::CraftingManager;
use oxide::event::EventBus;
use oxide::event::event::{BlockBreakEvent, PlayerChatEvent};
use oxide::scheduler::Scheduler;
use oxide::level::chunk::Chunk;
use oxide::level::generator::{FlatGenerator, NormalGenerator, Generator};
use oxide::protocol::packet::DataPacket;
use oxide::protocol::packets::*;
use oxide::protocol::types::Item;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[test]
fn test_crafting_recipes() {
    let cm = CraftingManager::default();
    assert!(cm.count() > 0, "Crafting manager should have default recipes");

    // Test shaped recipe matching (wood to planks: 1 wood in 1x1 grid)
    let grid_wood = vec![Item::new(17, 0, 1)];
    let match_wood = cm.match_shaped(&grid_wood, 1, 1);
    assert!(match_wood.is_some(), "Wood to planks recipe should match");
    let res = match_wood.unwrap().result;
    assert_eq!(res.id, 5, "Result should be oak planks");
    assert_eq!(res.count, 4, "Result count should be 4");

    // Test furnace recipe matching (iron ore -> iron ingot)
    let iron_ore = Item::new(15, 0, 1);
    let furnace_match = cm.match_furnace(&iron_ore);
    assert!(furnace_match.is_some(), "Iron ore smelting recipe should match");
    assert_eq!(furnace_match.unwrap().result.id, 265, "Smelting iron ore should produce iron ingot");
}

#[test]
fn test_block_and_item_registry() {
    let blocks = BlockRegistry::default();
    assert!(blocks.count() > 30, "Block registry should register default vanilla blocks");

    let bedrock = blocks.get(7).expect("Bedrock should be registered");
    assert_eq!(bedrock.id(), 7);
    assert!(bedrock.hardness() < 0.0, "Bedrock should be unbreakable");

    let diamond_ore = blocks.get(56).expect("Diamond Ore should be registered");
    assert_eq!(diamond_ore.id(), 56);
    assert_eq!(diamond_ore.hardness(), 3.0);
    assert_eq!(diamond_ore.tool_type(), oxide::block::ToolType::Pickaxe);

    let items = ItemRegistry::default();
    assert!(items.count() > 30, "Item registry should register default items");

    let diamond_pickaxe = items.get(278).expect("Diamond Pickaxe should be registered");
    assert_eq!(diamond_pickaxe.id(), 278);
    assert_eq!(diamond_pickaxe.durability(), 1561);
    assert_eq!(diamond_pickaxe.tool_type(), oxide::block::ToolType::Pickaxe);
}

#[test]
fn test_event_bus_and_cancellation() {
    let bus = EventBus::new();
    bus.on("BlockBreakEvent", |ev: &mut BlockBreakEvent| {
        if ev.block_id == 7 { // Cancel breaking bedrock
            ev.cancelled = true;
        }
    });

    let ev1 = bus.fire(BlockBreakEvent {
        player_name: "Tester".to_string(),
        x: 0, y: 0, z: 0,
        block_id: 7,
        cancelled: false,
    });
    assert!(ev1.cancelled, "Bedrock breaking should be cancelled by listener");

    let ev2 = bus.fire(BlockBreakEvent {
        player_name: "Tester".to_string(),
        x: 0, y: 10, z: 0,
        block_id: 1, // Stone
        cancelled: false,
    });
    assert!(!ev2.cancelled, "Stone breaking should not be cancelled");

    // Chat event
    bus.on("PlayerChatEvent", |ev: &mut PlayerChatEvent| {
        if ev.message.contains("badword") {
            ev.cancelled = true;
        }
    });
    let chat_ev = bus.fire(PlayerChatEvent {
        player_name: "Tester".to_string(),
        message: "hello badword test".to_string(),
        cancelled: false,
    });
    assert!(chat_ev.cancelled, "Chat with forbidden words should be cancelled");
}

#[test]
fn test_scheduler() {
    let sched = Scheduler::new();
    let once_ran = Arc::new(AtomicU32::new(0));
    let rep_count = Arc::new(AtomicU32::new(0));

    let once_clone = once_ran.clone();
    sched.schedule(5, move || {
        once_clone.fetch_add(1, Ordering::SeqCst);
    });

    let rep_clone = rep_count.clone();
    let rep_id = sched.schedule_repeating(2, 3, move |_tick| {
        rep_clone.fetch_add(1, Ordering::SeqCst);
    });

    sched.tick(1);
    assert_eq!(once_ran.load(Ordering::SeqCst), 0);
    assert_eq!(rep_count.load(Ordering::SeqCst), 0);

    sched.tick(2); // repeating runs
    assert_eq!(rep_count.load(Ordering::SeqCst), 1);
    assert_eq!(once_ran.load(Ordering::SeqCst), 0);

    sched.tick(5); // repeating runs (2+3=5), once runs (at 5)
    assert_eq!(rep_count.load(Ordering::SeqCst), 2);
    assert_eq!(once_ran.load(Ordering::SeqCst), 1);

    sched.cancel(rep_id);
    sched.tick(8);
    assert_eq!(rep_count.load(Ordering::SeqCst), 2, "Cancelled task should not run again");
}

#[test]
fn test_world_generators() {
    let flat = FlatGenerator::new();
    let mut chunk_flat = Chunk::new(0, 0);
    flat.generate_chunk(&mut chunk_flat);

    assert_eq!(chunk_flat.get_block_id(0, 0, 0), 7, "y=0 should be bedrock in flat world");
    assert_eq!(chunk_flat.get_block_id(0, 1, 0), 3, "y=1 should be dirt in flat world");
    assert_eq!(chunk_flat.get_block_id(0, 2, 0), 3, "y=2 should be dirt in flat world");
    assert_eq!(chunk_flat.get_block_id(0, 3, 0), 2, "y=3 should be grass in flat world");
    assert_eq!(chunk_flat.height_map[0], 4, "Height map for flat column should be 4");
    assert_eq!(chunk_flat.biome_ids[0], 1, "Biome should be Plains (1)");

    let normal = NormalGenerator::new(12345);
    let mut chunk_normal = Chunk::new(0, 0);
    normal.generate_chunk(&mut chunk_normal);
    normal.populate(&mut chunk_normal);

    assert_eq!(chunk_normal.get_block_id(0, 0, 0), 7, "y=0 should be bedrock in normal world");
    let mut has_stone = false;
    for y in 1..60 {
        if chunk_normal.get_block_id(0, y, 0) == 1 {
            has_stone = true;
            break;
        }
    }
    assert!(has_stone, "Normal generator should produce subterranean stone layer");
}

#[test]
fn test_packet_encode_decode() {
    // ContainerOpenPacket
    let open_pk = ContainerOpenPacket::new(3, 0, 100, 64, 200);
    let encoded = open_pk.encode();
    let decoded = ContainerOpenPacket::decode(encoded);
    assert_eq!(decoded.window_id, 3);
    assert_eq!(decoded.container_type, 0);
    assert_eq!(decoded.x, 100);
    assert_eq!(decoded.y, 64);
    assert_eq!(decoded.z, 200);

    // MobEquipmentPacket
    let item = Item::new(278, 5, 1);
    let mob_eq = MobEquipmentPacket::new(42, item.clone(), 2);
    let encoded = mob_eq.encode();
    let decoded = MobEquipmentPacket::decode(encoded);
    assert_eq!(decoded.entity_runtime_id, 42);
    assert_eq!(decoded.inventory_slot, 2);
    assert_eq!(decoded.hotbar_slot, 2);
    assert_eq!(decoded.item.id, 278);

    // StartGamePacket
    let start_pk = StartGamePacket::new(1001, 1001);
    let encoded = start_pk.encode();
    let decoded = StartGamePacket::decode(encoded);
    assert_eq!(decoded.entity_unique_id, 1001);
    assert_eq!(decoded.entity_runtime_id, 1001);
    assert_eq!(decoded.difficulty, 1);

    // FullChunkDataPacket with negative coordinates (ZigZag varint verification)
    let chunk_pk = FullChunkDataPacket::new(-7, -12, vec![1, 2, 3, 4]);
    let encoded = chunk_pk.encode();
    let decoded = FullChunkDataPacket::decode(encoded);
    assert_eq!(decoded.chunk_x, -7, "Negative chunk x must decode correctly");
    assert_eq!(decoded.chunk_z, -12, "Negative chunk z must decode correctly");
    assert_eq!(decoded.data, vec![1, 2, 3, 4]);
}

#[test]
fn test_zigzag_varint_negative_values() {
    use oxide::utils::Binary;

    let test_values = [0, -1, 1, -2, 2, -100, 100, -2147483648, 2147483647];
    for &val in &test_values {
        let bytes = Binary::write_var_int(val);
        let mut offset = 0usize;
        let decoded = Binary::read_var_int(&bytes, &mut offset).expect("VarInt decoding should succeed");
        assert_eq!(decoded, val, "Round-trip VarInt mismatch for {}", val);
        assert_eq!(offset, bytes.len(), "All bytes should be consumed");
    }

    // -1 in ZigZag is encoded as 1 (single byte 0x01)
    let minus_one = Binary::write_var_int(-1);
    assert_eq!(minus_one, vec![0x01], "-1 in ZigZag must encode as 1 single byte [0x01]");

    // -2 in ZigZag is encoded as 3 (single byte 0x03)
    let minus_two = Binary::write_var_int(-2);
    assert_eq!(minus_two, vec![0x03], "-2 in ZigZag must encode as 1 single byte [0x03]");

    let test_longs = [0i64, -1i64, 1i64, -100i64, i64::MIN, i64::MAX];
    for &val in &test_longs {
        let bytes = Binary::write_var_long(val);
        let mut offset = 0usize;
        let decoded = Binary::read_var_long(&bytes, &mut offset).expect("VarLong decoding should succeed");
        assert_eq!(decoded, val, "Round-trip VarLong mismatch for {}", val);
    }
}
