use oxide::block::BlockRegistry;
use oxide::command::CommandManager;
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

#[test]
fn test_creative_items_loading() {
    let items = oxide::item::creative::get_creative_items();
    assert!(!items.is_empty(), "Creative item palette must not be empty");
    assert_eq!(items.len(), 823, "MCPE 1.1.5 vanilla creative item count should be 823");
    // Verify first item is building block or tool (not all 0)
    assert!(items.iter().any(|item| item.id > 0));
}

#[test]
fn test_available_commands_packet() {
    let cm = CommandManager::new();
    let packet = cm.generate_available_commands_packet();
    assert!(!packet.commands.is_empty());
    let parsed: serde_json::Value = serde_json::from_str(&packet.commands).expect("Command palette must be valid JSON");
    assert!(parsed.get("help").is_some(), "Must contain help command");
    assert!(parsed.get("gamemode").is_some(), "Must contain gamemode command");
    assert!(parsed.get("tp").is_some() || parsed.get("teleport").is_some(), "Must contain teleport command");
}

#[test]
fn test_add_item_entity_packet_with_velocity() {
    use oxide::protocol::packets::AddItemEntityPacket;
    use oxide::protocol::packet::DataPacket;
    use oxide::protocol::types::Item;

    let mut pk = AddItemEntityPacket::new(1001, Item::new(1, 0, 1), 10.5, 64.0, -15.5);
    pk.speed_x = 0.05;
    pk.speed_y = 0.2;
    pk.speed_z = -0.08;

    let encoded = pk.encode();
    let decoded = AddItemEntityPacket::decode(encoded);

    assert_eq!(decoded.entity_runtime_id, 1001);
    assert_eq!(decoded.item.id, 1);
    assert!((decoded.x - 10.5).abs() < 1e-4);
    assert!((decoded.y - 64.0).abs() < 1e-4);
    assert!((decoded.z - (-15.5)).abs() < 1e-4);
    assert!((decoded.speed_x - 0.05).abs() < 1e-4);
    assert!((decoded.speed_y - 0.2).abs() < 1e-4);
    assert!((decoded.speed_z - (-0.08)).abs() < 1e-4);
}

#[test]
fn test_projectile_types_and_constants() {
    use oxide::entity::projectile::ProjectileKind;

    assert_eq!(ProjectileKind::EnderPearl.network_entity_type(), 87);
    assert_eq!(ProjectileKind::Snowball.network_entity_type(), 81);
    assert_eq!(ProjectileKind::Egg.network_entity_type(), 82);
    assert_eq!(ProjectileKind::Arrow.network_entity_type(), 80);
    assert_eq!(ProjectileKind::ExpBottle.network_entity_type(), 68);

    assert!((ProjectileKind::EnderPearl.gravity() - 0.03).abs() < 1e-6);
    assert!((ProjectileKind::Arrow.gravity() - 0.05).abs() < 1e-6);
    assert!((ProjectileKind::EnderPearl.drag() - 0.01).abs() < 1e-6);
}

#[tokio::test]
async fn test_liquid_simulation_spread() {
    use oxide::level::{World, LiquidSimulation};
    use oxide::raknet::RakNetServer;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::collections::HashMap;

    let world = Arc::new(RwLock::new(World::new()));
    // Set up world blocks: air at (5, 5, 5), air below at (5, 4, 5)
    {
        let mut w = world.write();
        w.set_block(5, 5, 5, 8, 0); // Water source
        w.set_block(5, 4, 5, 0, 0); // Air below
    }

    let liquid = LiquidSimulation::new();
    liquid.schedule_update(5, 5, 5);

    let rak = Arc::new(RakNetServer::new("127.0.0.1:0".parse().unwrap(), "Oxide".to_string()).await.unwrap());
    let players = Arc::new(RwLock::new(HashMap::new()));

    liquid.tick(&world, &rak, &players).await;

    // Water should flow down into (5, 4, 5)
    let below_id = {
        let mut w = world.write();
        w.get_block_id(5, 4, 5)
    };
    assert_eq!(below_id, 8, "Water should flow downward into empty air block");
}

#[tokio::test]
async fn test_liquid_horizontal_spread() {
    use oxide::level::{World, LiquidSimulation};
    use oxide::raknet::RakNetServer;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::collections::HashMap;

    let world = Arc::new(RwLock::new(World::new()));
    // Set up ground at y=3 (solid dirt = 3), water source at (10, 4, 10), air surrounding at y=4
    {
        let mut w = world.write();
        w.set_block(10, 3, 10, 3, 0); // Solid ground directly below
        w.set_block(9, 3, 10, 3, 0);
        w.set_block(11, 3, 10, 3, 0);
        w.set_block(10, 3, 9, 3, 0);
        w.set_block(10, 3, 11, 3, 0);

        w.set_block(10, 4, 10, 8, 0); // Water source
        w.set_block(9, 4, 10, 0, 0);  // Air to the west
        w.set_block(11, 4, 10, 0, 0); // Air to the east
        w.set_block(10, 4, 9, 0, 0);  // Air to the north
        w.set_block(10, 4, 11, 0, 0); // Air to the south
    }

    let liquid = LiquidSimulation::new();
    liquid.schedule_update(10, 4, 10);

    let rak = Arc::new(RakNetServer::new("127.0.0.1:0".parse().unwrap(), "Oxide".to_string()).await.unwrap());
    let players = Arc::new(RwLock::new(HashMap::new()));

    liquid.tick(&world, &rak, &players).await;

    // Verify all 4 horizontal neighbors became water (8) with decay 1
    let mut w = world.write();
    assert_eq!(w.get_block_id(9, 4, 10), 8);
    assert_eq!(w.get_block_id(11, 4, 10), 8);
    assert_eq!(w.get_block_id(10, 4, 9), 8);
    assert_eq!(w.get_block_id(10, 4, 11), 8);
}

#[tokio::test]
async fn test_liquid_lava_spread() {
    use oxide::level::{World, LiquidSimulation};
    use oxide::raknet::RakNetServer;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::collections::HashMap;

    let world = Arc::new(RwLock::new(World::new()));
    // Set up ground at y=3 (solid stone = 1), lava source at (20, 4, 20), air surrounding at y=4
    {
        let mut w = world.write();
        w.set_block(20, 3, 20, 1, 0); // Solid ground
        w.set_block(19, 3, 20, 1, 0);
        w.set_block(21, 3, 20, 1, 0);
        w.set_block(20, 3, 19, 1, 0);
        w.set_block(20, 3, 21, 1, 0);

        w.set_block(20, 4, 20, 10, 0); // Lava source
        w.set_block(19, 4, 20, 0, 0);
        w.set_block(21, 4, 20, 0, 0);
        w.set_block(20, 4, 19, 0, 0);
        w.set_block(20, 4, 21, 0, 0);
    }

    let liquid = LiquidSimulation::new();
    liquid.schedule_update(20, 4, 20);

    let rak = Arc::new(RakNetServer::new("127.0.0.1:0".parse().unwrap(), "Oxide".to_string()).await.unwrap());
    let players = Arc::new(RwLock::new(HashMap::new()));

    liquid.tick(&world, &rak, &players).await;

    // Verify all 4 horizontal neighbors became lava (10)
    let mut w = world.write();
    assert_eq!(w.get_block_id(19, 4, 20), 10);
    assert_eq!(w.get_block_id(21, 4, 20), 10);
    assert_eq!(w.get_block_id(20, 4, 19), 10);
    assert_eq!(w.get_block_id(20, 4, 21), 10);
}

#[test]
fn test_gamemode_command_parsing() {
    use oxide::command::CommandManager;

    let cmd_mgr = CommandManager::new();
    assert!(cmd_mgr.count() > 0);

    // Test that gamemode command is registered
    let cmds = cmd_mgr.all_commands();
    assert!(cmds.iter().any(|c| c.name() == "gamemode"));
}

#[test]
fn test_plugin_manager_lifecycle() {
    use oxide::plugin::{PluginManager, Plugin, PluginDescription};

    struct TestPlugin {
        desc: PluginDescription,
        enabled: bool,
    }
    impl Plugin for TestPlugin {
        fn on_enable(&mut self) { self.enabled = true; }
        fn on_disable(&mut self) { self.enabled = false; }
        fn description(&self) -> &PluginDescription { &self.desc }
    }

    let pm = PluginManager::new();
    assert_eq!(pm.count(), 0);

    let desc = PluginDescription::new("TestPL", "1.0.0");
    pm.enable(Box::new(TestPlugin { desc, enabled: false }));

    assert_eq!(pm.count(), 1);
    assert_eq!(pm.list(), vec!["TestPL".to_string()]);

    pm.disable("TestPL");
    assert_eq!(pm.count(), 0);
}

#[test]
fn test_level_sound_event_with_extra() {
    use oxide::protocol::packets::{LevelSoundEventPacket, level_event};

    let pk = LevelSoundEventPacket::new_with_extra(level_event::SOUND_BREAK, 10.0, 64.0, 20.0, 2);
    assert_eq!(pk.sound, level_event::SOUND_BREAK);
    assert_eq!(pk.extra_data, 2);
    assert_eq!(pk.x, 10.0);
    assert_eq!(pk.y, 64.0);
    assert_eq!(pk.z, 20.0);
}

#[test]
fn test_expanded_catalog() {
    let blocks = BlockRegistry::default();
    assert!(blocks.count() >= 80, "Block registry should have 80+ blocks, got {}", blocks.count());

    let items = ItemRegistry::default();
    assert!(items.count() >= 60, "Item registry should have 60+ items, got {}", items.count());
}

#[test]
fn test_hell_and_ender_generators() {
    use oxide::level::generator::{HellGenerator, EnderGenerator};

    // 1. Hell Generator test
    let hell = HellGenerator::new(12345);
    let mut hell_chunk = Chunk::new(0, 0);
    hell.generate_chunk(&mut hell_chunk);
    hell.populate(&mut hell_chunk);

    assert_eq!(hell_chunk.get_block_id(0, 0, 0), 7, "Hell bottom must be bedrock");
    assert_eq!(hell_chunk.get_block_id(0, 127, 0), 7, "Hell ceiling must be bedrock");
    assert_eq!(hell_chunk.biome_ids[0], 8, "Hell biome must be 8");

    // Check presence of netherrack (87)
    let mut has_netherrack = false;
    for y in 10..120 {
        if hell_chunk.get_block_id(8, y, 8) == 87 {
            has_netherrack = true;
            break;
        }
    }
    assert!(has_netherrack, "Hell chunk should contain netherrack");

    // 2. Ender Generator test
    let ender = EnderGenerator::new(12345);
    let mut end_chunk = Chunk::new(0, 0);
    ender.generate_chunk(&mut end_chunk);
    ender.populate(&mut end_chunk);

    assert_eq!(end_chunk.biome_ids[0], 9, "End biome must be 9");

    let mut has_endstone = false;
    for y in 45..70 {
        if end_chunk.get_block_id(8, y, 8) == 121 {
            has_endstone = true;
            break;
        }
    }
    assert!(has_endstone, "Center End chunk should contain end stone");
}

#[test]
fn test_expanded_crafting_recipes() {
    let cm = CraftingManager::default();
    assert!(cm.count() >= 45, "Crafting manager should have at least 45 recipes, got {}", cm.count());

    // Bow recipe (280 + 287)
    let bow_recipe = cm.get_recipe("craft_bow");
    assert!(bow_recipe.is_some(), "Bow recipe should exist");
    assert_eq!(bow_recipe.unwrap().result.id, 261);

    // Arrow recipe (318 + 280 + 288)
    let arrow_recipe = cm.get_recipe("craft_arrow");
    assert!(arrow_recipe.is_some(), "Arrow recipe should exist");
    let arrow = arrow_recipe.unwrap();
    assert_eq!(arrow.result.id, 262);
    assert_eq!(arrow.result.count, 4);

    // TNT recipe (289 + 12)
    let tnt_recipe = cm.get_recipe("craft_tnt");
    assert!(tnt_recipe.is_some(), "TNT recipe should exist");
    assert_eq!(tnt_recipe.unwrap().result.id, 46);

    // Golden apple recipe (266 + 260)
    let gapple_recipe = cm.get_recipe("craft_golden_apple");
    assert!(gapple_recipe.is_some(), "Golden apple recipe should exist");
    assert_eq!(gapple_recipe.unwrap().result.id, 322);

    // Bread recipe
    let bread_recipe = cm.get_recipe("craft_bread");
    assert!(bread_recipe.is_some(), "Bread recipe should exist");
    assert_eq!(bread_recipe.unwrap().result.id, 297);
}

#[test]
fn test_dimension_command_and_packets() {
    let cmd_mgr = CommandManager::new();
    let cmds = cmd_mgr.all_commands();
    assert!(cmds.iter().any(|c| c.name() == "dimension"), "Dimension command should be registered");

    // Test ChangeDimensionPacket serialization
    let pk = ChangeDimensionPacket::new(1, 100.0, 64.0, 200.0);
    assert_eq!(pk.dimension, 1);
    assert_eq!(pk.x, 100.0);
    assert_eq!(pk.y, 64.0);
    assert_eq!(pk.z, 200.0);

    let mut stream = oxide::utils::BinaryStream::new();
    pk.encode_payload(&mut stream);
    assert!(stream.len() > 0);
}

#[test]
fn test_mob_ai_math_and_yaw_pitch() {
    use oxide::entity::manager::{calculate_yaw, calculate_pitch};

    // Yaw angles: 0 is south (+z), 90 is west (-x), 180 is north (-z), 270 is east (+x)
    let yaw_north = calculate_yaw(0.0, -10.0);
    assert!((yaw_north.abs() - 180.0).abs() < 1.0 || yaw_north.abs() < 1.0);

    let yaw_east = calculate_yaw(10.0, 0.0);
    assert_eq!(yaw_east, -90.0);

    let yaw_west = calculate_yaw(-10.0, 0.0);
    assert_eq!(yaw_west, 90.0);

    // Pitch angles: looking straight ahead is 0, up is negative, down is positive
    let pitch_level = calculate_pitch(0.0, 64.0, 0.0, 10.0, 64.0, 0.0);
    assert_eq!(pitch_level, 0.0);

    let pitch_up = calculate_pitch(0.0, 64.0, 0.0, 0.0, 74.0, 0.0);
    assert!(pitch_up < -80.0);

    let pitch_down = calculate_pitch(0.0, 64.0, 0.0, 0.0, 54.0, 0.0);
    assert!(pitch_down > 80.0);
}

#[test]
fn test_mob_loot_tables_vanilla_parity() {
    use oxide::entity::LootTable;

    // Test Zombie drops (Rotten flesh: 367)
    let zombie_drops = LootTable::get_drops(32, true, false, 0);
    for item in &zombie_drops {
        assert!(item.id == 367 || item.id == 265 || item.id == 391 || item.id == 392);
    }

    // Test Skeleton drops (Arrow: 262, Bone: 352)
    let skeleton_drops = LootTable::get_drops(34, true, false, 0);
    for item in &skeleton_drops {
        assert!(item.id == 262 || item.id == 352);
    }

    // Test Wither drops guaranteed Nether Star (399)
    let wither_drops = LootTable::get_drops(52, true, false, 0);
    assert_eq!(wither_drops.len(), 1);
    assert_eq!(wither_drops[0].id, 399);

    // Test Evoker drops guaranteed Totem of Undying (449)
    let evoker_drops = LootTable::get_drops(58, true, false, 0);
    assert!(evoker_drops.iter().any(|i| i.id == 449));
}

#[test]
fn test_effect_container_and_mechanics() {
    use oxide::entity::{EffectContainer, ActiveEffect, effect_id};

    let mut container = EffectContainer::new();
    container.add_effect(ActiveEffect::new(effect_id::SPEED, 200, 1));
    assert!(container.has_effect(effect_id::SPEED));
    assert_eq!(container.speed_multiplier(), 1.4); // Level 2 speed (+40%)

    // Add absorption
    container.add_effect(ActiveEffect::new(effect_id::ABSORPTION, 1000, 0));
    assert_eq!(container.absorption_hp, 4.0);

    // Add fire resistance
    container.add_effect(ActiveEffect::new(effect_id::FIRE_RESISTANCE, 500, 0));
    assert!(container.has_fire_resistance());

    // Tick effects
    let (expired, _) = container.tick();
    assert!(expired.is_empty());
    assert_eq!(container.get_effect(effect_id::SPEED).unwrap().duration, 199);
}

#[test]
fn test_hotbar_mapping_and_inventory_wire() {
    use oxide::inventory::inventories::PlayerInventory;
    use oxide::protocol::types::Item;

    let inv = PlayerInventory::new();
    inv.set_item(12, Item::new(276, 0, 1));
    inv.set_held_index_with_mapping(2, 12 + 9);
    assert_eq!(inv.held_slot(), 12);
    assert_eq!(inv.held_item().id, 276);

    let links = inv.wire_hotbar_links();
    assert_eq!(links[2], 12 + 9);

    inv.set_armor(0, Item::new(310, 0, 1));
    inv.set_offhand(Item::new(450, 0, 1));
    assert_eq!(inv.helmet().id, 310);
    assert_eq!(inv.offhand().id, 450);
}

#[test]
fn test_block_placement_drops_and_lighting() {
    use oxide::block::physics::{get_block_drops_ext, resolve_placement, PlacementResult};
    use oxide::item::enchantment::{apply_enchantment_to_item, TYPE_MINING_SILK_TOUCH};
    use oxide::level::{World, light};
    use oxide::protocol::types::Item;

    let mut world = World::new();
    // Place slab at (5,4,5) then merge into double slab (43)
    let first = resolve_placement(&mut world, 5, 3, 5, 1, 0.2, 0.0, 44, 2);
    if let PlacementResult::Single { x, y, z, id, meta } = first {
        world.set_block(x, y, z, id, meta);
        assert_eq!(id, 44);
    } else {
        panic!("expected single slab placement");
    }
    let merge = resolve_placement(&mut world, 5, 4, 5, 1, 0.2, 0.0, 44, 2);
    if let PlacementResult::Single { id, .. } = merge {
        assert_eq!(id, 43);
    } else {
        panic!("expected slab merge into 43");
    }

    // Silk Touch drops Diamond Ore (56) instead of Diamond (264)
    let mut silk_pick = Item::new(278, 0, 1);
    apply_enchantment_to_item(&mut silk_pick, TYPE_MINING_SILK_TOUCH, 1);
    let silk_drops = get_block_drops_ext(56, 0, &silk_pick);
    assert_eq!(silk_drops[0].id, 56);

    let plain_pick = Item::new(257, 0, 1);
    let plain_drops = get_block_drops_ext(56, 0, &plain_pick);
    assert_eq!(plain_drops[0].id, 264);

    // Block lighting BFS from Torch (50)
    world.set_block(10, 5, 10, 50, 5);
    light::update_light_around(&mut world, 10, 5, 10);
    assert!(world.get_full_light_opt(10, 5, 10) >= 14);
    assert!(world.get_full_light_opt(12, 5, 10) >= 12);
}

#[test]
fn test_explosion_raycast_and_tnt() {
    use oxide::level::{World, explosion};

    let mut world = World::new();
    world.set_block(10, 4, 10, 1, 0);   // Stone (destructible)
    world.set_block(11, 4, 10, 46, 0);  // TNT (chains)
    world.set_block(9, 4, 10, 49, 0);   // Obsidian (blast resistant)

    let res = explosion::compute_explosion(&mut world, 10.5, 4.5, 10.5, 4.0);
    assert_eq!(world.get_block_id(10, 4, 10), 0);
    assert_eq!(world.get_block_id(9, 4, 10), 49);
    assert!(res.ignited_tnts.iter().any(|&(tx, ty, tz, _)| {
        (tx.floor() as i32, ty.floor() as i32, tz.floor() as i32) == (11, 4, 10)
    }));
}

#[test]
fn test_armor_damage_reduction_and_totem() {
    use oxide::entity::damage::{apply_player_damage, DamageCause};
    use oxide::protocol::handler::PlayerState;
    use oxide::protocol::types::Item;

    let mut naked = PlayerState::new();
    naked.gamemode = 0;
    naked.spawned = true;
    let d_naked = apply_player_damage(&mut naked, 10.0, DamageCause::EntityAttack, false);
    assert!((d_naked.final_damage - 10.0).abs() < 0.01);
    assert_eq!(naked.health, 10);

    let mut armored = PlayerState::new();
    armored.gamemode = 0;
    armored.spawned = true;
    armored.inventory.set_armor(0, Item::new(310, 0, 1)); // Diamond helmet (3)
    armored.inventory.set_armor(1, Item::new(311, 0, 1)); // Diamond chest (8)
    armored.inventory.set_armor(2, Item::new(312, 0, 1)); // Diamond legs (6)
    armored.inventory.set_armor(3, Item::new(313, 0, 1)); // Diamond boots (3) => 20 pts = 80% reduction
    let d_armored = apply_player_damage(&mut armored, 10.0, DamageCause::EntityAttack, false);
    assert!((d_armored.final_damage - 2.0).abs() < 0.01);
    assert_eq!(armored.health, 18);

    // Totem of Undying resurrection from fatal hit
    armored.inventory.set_offhand(Item::new(450, 0, 1));
    let lethal = apply_player_damage(&mut armored, 100.0, DamageCause::EntityAttack, false);
    assert!(lethal.totem_used);
    assert!(!lethal.died);
    assert_eq!(armored.health, 1);
    assert!(armored.inventory.offhand().is_air());
}

#[test]
fn test_potion_metadata_and_brewing_recipes() {
    use oxide::entity::effect::{get_potion_effects, effect_id};
    use oxide::inventory::crafting::CraftingManager;
    use oxide::protocol::types::Item;

    assert_eq!(get_potion_effects(5)[0].id, effect_id::NIGHT_VISION);
    assert_eq!(get_potion_effects(21)[0].id, effect_id::INSTANT_HEALTH);
    assert_eq!(get_potion_effects(28)[0].id, effect_id::REGENERATION);
    assert_eq!(get_potion_effects(31)[0].id, effect_id::STRENGTH);

    let cm = CraftingManager::default();
    let awkward = cm.match_brewing(&Item::new(372, 0, 1), &Item::new(373, 0, 1));
    assert_eq!(awkward.map(|i| (i.id, i.meta)), Some((373, 4)));

    let str_pot = cm.match_brewing(&Item::new(377, 0, 1), &Item::new(373, 4, 1));
    assert_eq!(str_pot.map(|i| (i.id, i.meta)), Some((373, 31)));

    let splash = cm.match_brewing(&Item::new(289, 0, 1), &Item::new(373, 31, 1));
    assert_eq!(splash.map(|i| (i.id, i.meta)), Some((438, 31)));
}

#[test]
fn test_double_chest_furnace_brewing_anvil() {
    use oxide::tile::TileManager;
    use oxide::inventory::crafting::CraftingManager;
    use oxide::item::enchantment::anvil_combine_items;
    use oxide::protocol::types::Item;
    use std::net::SocketAddr;

    let tm = TileManager::new();
    let addr: SocketAddr = "127.0.0.1:40000".parse().unwrap();

    // Double chest 54 slots
    tm.ensure_chest_in_dim(0, 10, 4, 10, false);
    tm.ensure_chest_in_dim(0, 11, 4, 10, false);
    assert_eq!(tm.try_pair_chest_in_dim(0, 11, 4, 10, false), Some((10, 4, 10)));
    let (slots, _) = tm.open_chest_in_dim(addr, 0, 10, 4, 10);
    assert_eq!(slots.len(), 54);

    // Furnace lava bucket leaves empty bucket residue
    let cm = CraftingManager::default();
    tm.ensure_furnace_in_dim(0, 20, 4, 20);
    tm.open_furnace_in_dim(addr, 0, 20, 4, 20);
    tm.set_furnace_slot_in_dim(0, 20, 4, 20, 0, Item::new(15, 0, 1)); // Iron Ore
    tm.set_furnace_slot_in_dim(0, 20, 4, 20, 1, Item::new(325, 10, 1)); // Lava Bucket
    let fur_up = tm.tick_furnaces_live(&cm);
    assert_eq!(fur_up.block_toggles, vec![(0, 20, 4, 20, true)]);
    let items = tm.open_furnace_in_dim(addr, 0, 20, 4, 20);
    assert_eq!((items[1].id, items[1].meta), (325, 0));

    // Anvil repair + rename
    let damaged_sword = Item::new(276, 500, 1);
    let (repaired, cost) = anvil_combine_items(&damaged_sword, &Item::new(264, 0, 1), Some("Excalibur")).unwrap();
    assert!(repaired.meta < 500);
    assert!(cost >= 1);
}

#[test]
fn test_entity_aabb_collision_and_xp_orbs() {
    use oxide::entity::physics::{move_entity_with_collision, block_collision_box};
    use oxide::entity::manager::split_xp_orb_sizes;
    use oxide::level::World;

    let mut world = World::new();
    // Solid floor at y=3, wall at x=5, y=4..6
    world.set_block(3, 3, 3, 1, 0);
    world.set_block(4, 3, 3, 1, 0);
    world.set_block(5, 4, 3, 1, 0);
    world.set_block(5, 5, 3, 1, 0);

    let res = move_entity_with_collision(&world, 4.4, 4.0, 3.5, 0.5, 0.0, 0.0, 0.3, 1.8, 0.5, true);
    assert!(res.collided_horizontally);
    assert!(res.x <= 4.7001);

    // Fence has 1.5 block height collision
    let fence_bb = block_collision_box(85, 0, 10, 4, 10).unwrap();
    assert_eq!(fence_bb.max_y, 5.5);

    // XP splitting
    let orbs = split_xp_orb_sizes(25);
    assert_eq!(orbs.iter().sum::<u32>(), 25);
    assert_eq!(orbs, vec![17, 7, 1]);
}

#[test]
fn test_world_chunk_tile_and_player_persistence() {
    use oxide::level::{World, storage};
    use oxide::tile::TileManager;
    use oxide::protocol::handler::PlayerState;
    use oxide::protocol::types::Item;
    use std::net::SocketAddr;

    let tmp_root = std::env::temp_dir().join("oxide_persist_test");
    let _ = std::fs::remove_dir_all(&tmp_root);
    let wdir = tmp_root.join("worlds").join("world");

    {
        let mut world = World::with_generator_and_dir("flat", 42, Some(wdir.clone()), "world");
        world.time = 1234;
        world.set_block(7, 10, 7, 57, 0); // Diamond block
        let saved = world.save_all();
        assert_eq!(saved, 1);
    }

    {
        let mut world2 = World::with_generator_and_dir("flat", 99, Some(wdir.clone()), "world");
        assert_eq!(world2.time, 1234);
        assert_eq!(world2.seed, 42);
        assert_eq!(world2.get_block_id(7, 10, 7), 57);
    }

    // Tile persistence
    let tm1 = TileManager::new();
    let addr: SocketAddr = "127.0.0.1:41000".parse().unwrap();
    tm1.ensure_chest_in_dim(1, 2, 64, 2, false);
    tm1.open_chest_in_dim(addr, 1, 2, 64, 2);
    tm1.set_chest_slot_in_dim(1, 2, 64, 2, 0, Item::new(264, 0, 32));
    tm1.set_sign_text_in_dim(0, 3, 4, 3, ["Line 1".into(), "Line 2".into(), "".into(), "".into()], "Alex".into());
    let tile_bytes = tm1.export_tiles_bytes();

    let tm2 = TileManager::new();
    assert!(tm2.import_tiles_bytes(&tile_bytes));
    let items2 = tm2.open_chest_in_dim(addr, 1, 2, 64, 2).0;
    assert_eq!((items2[0].id, items2[0].count), (264, 32));
    assert_eq!(tm2.get_sign_text(3, 4, 3).unwrap()[0], "Line 1");

    // Player .dat persistence
    let mut ps = PlayerState::new();
    ps.username = "TesterOne".into();
    ps.x = 12.5;
    ps.y = 70.0;
    ps.z = -4.5;
    ps.dimension = 1;
    ps.xp_level = 9;
    ps.inventory.set_item(3, Item::new(276, 0, 1));
    ps.inventory.set_armor(1, Item::new(311, 0, 1));
    storage::save_player_dat(&tmp_root, &ps).unwrap();

    let loaded = storage::load_player_dat(&tmp_root, "testerone").unwrap();
    assert_eq!(loaded.dimension, 1);
    assert_eq!(loaded.xp_level, 9);
    assert_eq!(loaded.inventory[3].id, 276);
    assert_eq!(loaded.armor[1].id, 311);

    let _ = std::fs::remove_dir_all(&tmp_root);
}

#[test]
fn test_redstone_wire_torch_lamp_door_and_tnt() {
    use oxide::level::{World, redstone};

    let mut world = World::new();
    // Floor stone y=3
    for x in 0..=8 {
        world.set_block(x, 3, 5, 1, 0);
    }
    // Activated Lever (69, meta 5|8 = 13) at (1,4,5) -> Wires (55) at (2..=4,4,5) -> Lamp (123) at (5,4,5) & Iron Door (71) at (3,4..=5,6)
    world.set_block(1, 4, 5, 69, 5 | 0x08);
    world.set_block(2, 4, 5, 55, 0);
    world.set_block(3, 4, 5, 55, 0);
    world.set_block(4, 4, 5, 55, 0);
    world.set_block(5, 4, 5, 123, 0);
    world.set_block(3, 4, 6, 71, 0);
    world.set_block(3, 5, 6, 71, 8);

    // Torch inverter test: stone at (2,4,2), active torch (76) on side (3,4,2), power stone with redstone block (152) at (1,4,2)
    world.set_block(1, 4, 2, 152, 0);
    world.set_block(2, 4, 2, 1, 0);
    world.set_block(3, 4, 2, 76, 1);

    // TNT next to wire at (4,4,6)
    world.set_block(4, 4, 6, 46, 0);

    let res = redstone::update_redstone_around(&mut world, &[(1, 4, 5), (1, 4, 2)]);
    assert_eq!(world.get_block_data_opt(2, 4, 5), 15);
    assert_eq!(world.get_block_data_opt(3, 4, 5), 14);
    assert_eq!(world.get_block_data_opt(4, 4, 5), 13);
    assert_eq!(world.get_block_id(5, 4, 5), 124); // Lamp lit
    assert_ne!(world.get_block_data_opt(3, 4, 6) & 0x04, 0); // Iron Door opened
    assert_eq!(world.get_block_id(3, 4, 2), 75); // Inverted torch turned off
    assert!(res.ignited_tnts.iter().any(|&(tx, ty, tz, _)| (tx, ty, tz) == (4, 4, 6)));
}


